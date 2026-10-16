import { test, expect } from '@playwright/test';
import { setup, exported } from '../tests/fixtures.ts';
import { ready, download } from './helpers.ts';
import { screenshotInk } from './screenshot-pixels.ts';
test('WEB-PRECISION-001 actual giant portable stroke renders across the viewport and roundtrips unchanged', async ({
  page,
}, info) => {
  const { engine, work } = await setup();
  let bytes: Uint8Array;
  try {
    engine.brush([1, 0, 0, 0.5], 12);
    engine.begin(1n, 2);
    engine.append(1n, [
      { x: -999999000, y: 100, time: 0, id: 1n },
      { x: 999999000, y: 100, time: 1, id: 2n },
    ]);
    engine.commit(1n);
    bytes = await exported(engine, work);
  } finally {
    await work.shutdown();
  }
  await ready(page);
  const chooser = page.waitForEvent('filechooser');
  await page.locator('#open').click();
  await (
    await chooser
  ).setFiles({
    name: 'giant.nsketch',
    mimeType: 'application/octet-stream',
    buffer: Buffer.from(bytes),
  });
  await expect(page.locator('#status')).toContainText('1 stroke');
  await expect(page.locator('#status')).not.toContainText('Rendering unavailable');
  await page.locator('#width').focus();
  const ink = await screenshotInk(page, 4);
  expect(ink.left).toBe(4);
  expect(ink.right).toBe(ink.width - 5);
  expect(ink.top).toBeGreaterThanOrEqual(93);
  expect(ink.bottom).toBeLessThanOrEqual(107);
  expect((await download(page, info, 'giant-roundtrip.nsketch')).bytes).toEqual(Buffer.from(bytes));
  await info.attach('shared-prepared-giant-ui', {
    body: await page.screenshot(),
    contentType: 'image/png',
  });
});
