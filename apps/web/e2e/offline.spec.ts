import { test, expect } from '@playwright/test';
import { readFile, writeFile } from 'node:fs/promises';
import { join } from 'node:path';
import { ready, stroke, inkPixels } from './helpers.ts';
import type { Page } from '@playwright/test';
async function controlled(page: Page) {
  await ready(page);
  await page.evaluate(async () => {
    await navigator.serviceWorker.ready;
  });
  await page.reload();
  await expect
    .poll(() => page.evaluate(() => Boolean(navigator.serviceWorker.controller)))
    .toBe(true);
  await expect(page.locator('#status')).toContainText('0 strokes');
}
test('WEB-OFFLINE-001 real service worker reloads all production modules offline', async ({
  page,
  context,
}) => {
  await controlled(page);
  const cached = await page.evaluate(async () => {
    const cache = await caches.open(
      (await caches.keys()).find((key) => key.startsWith('native-sketch-'))!,
    );
    return (await cache.keys()).map((request) => new URL(request.url).pathname);
  });
  expect(cached).toContain('/nativesketch/sketch_wasm.wasm');
  expect(cached).toContain('/nativesketch/pointer-down.js');
  expect(cached.every((path) => path.startsWith('/nativesketch/'))).toBe(true);
  await context.setOffline(true);
  await page.reload();
  await expect(page.locator('#status')).toContainText('0 strokes');
  await stroke(page);
  await expect(page.locator('#status')).toContainText('1 stroke');
  expect(await inkPixels(page)).toBeGreaterThan(100);
});
test('WEB-WORKER-001 worker update activates and failed asset update retains working offline app', async ({
  page,
  context,
  baseURL,
}) => {
  const directory = process.env.SKETCH_BROWSER_SERVED;
  if (!directory) throw new Error('Missing disposable served fixture');
  const path = join(directory, 'sw.js');
  const original = await readFile(path, 'utf8');
  await controlled(page);
  const oldName = await page.evaluate(async () =>
    (await caches.keys()).find((key) => key.startsWith('native-sketch-'))!,
  );
  const updatedName = `${oldName}-e2e-update`;
  try {
    await writeFile(path, original.replace(oldName, updatedName));
    await page.evaluate(async () => {
      await (await navigator.serviceWorker.getRegistration())!.update();
    });
    await expect
      .poll(() =>
        page.evaluate(async () =>
          Boolean((await navigator.serviceWorker.getRegistration())?.waiting),
        ),
      )
      .toBe(true);
    await page.close();
    const next = await context.newPage();
    await next.goto(baseURL!);
    await expect.poll(() => next.evaluate(() => caches.keys())).toEqual([updatedName]);
    await expect(next.locator('#status')).toContainText('0 strokes');
    const brokenName = `${updatedName}-missing`;
    await writeFile(
      path,
      original
        .replace(oldName, brokenName)
        .replace('const ASSETS = [', 'const ASSETS = ["./missing-e2e-module.js",'),
    );
    await next.evaluate(async () => {
      await (await navigator.serviceWorker.getRegistration())!.update();
    });
    await expect.poll(() => next.evaluate(() => caches.keys())).toContain(brokenName);
    await expect
      .poll(() =>
        next.evaluate(async () => {
          const registration = await navigator.serviceWorker.getRegistration();
          return !registration?.installing && !registration?.waiting;
        }),
      )
      .toBe(true);
    expect(await next.evaluate(() => caches.keys())).toContain(updatedName);
    await context.setOffline(true);
    await next.reload();
    await expect(next.locator('#status')).toContainText('0 strokes');
    await stroke(next);
    expect(await inkPixels(next)).toBeGreaterThan(100);
  } finally {
    await writeFile(path, original);
  }
});
