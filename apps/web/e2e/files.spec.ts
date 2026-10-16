import { test, expect } from '@playwright/test';
import { ready, stroke, download, inkPixels } from './helpers.ts';
test('WEB-FILE-001 download remains dirty upload reopens exact portable drawing', async ({
  page,
}, info) => {
  await ready(page);
  await stroke(page);
  const saved = await download(page, info, 'drawing.nsketch');
  await expect(page.locator('#status')).toContainText('Unsaved');
  await page.locator('#new').click();
  await page.getByRole('button', { name: 'Discard and continue' }).click();
  await expect(page.locator('#status')).toContainText('0 strokes');
  const chooser = page.waitForEvent('filechooser');
  await page.locator('#open').click();
  await (await chooser).setFiles(saved.path);
  await expect(page.locator('#status')).toContainText('1 stroke');
  await expect(page.locator('#status')).not.toContainText('Unsaved');
  expect(await inkPixels(page)).toBeGreaterThan(100);
  expect((await download(page, info, 'reopened.nsketch')).bytes).toEqual(saved.bytes);
  const corruptChooser = page.waitForEvent('filechooser');
  await page.locator('#open').click();
  await (
    await corruptChooser
  ).setFiles({
    name: 'bad.nsketch',
    mimeType: 'application/octet-stream',
    buffer: Buffer.from('corrupt'),
  });
  await expect(page.locator('#error')).toBeVisible();
  await page.locator('#error').getByRole('button', { name: 'OK' }).click();
  expect((await download(page, info, 'after-corrupt.nsketch')).bytes).toEqual(saved.bytes);
  await expect(page.locator('#status')).not.toContainText('Unsaved');
});
test('WEB-GUARD-001 Escape and Cancel preserve dirty drawing and clear stale choice', async ({
  page,
}, info) => {
  await ready(page);
  await stroke(page);
  await page.locator('#new').click();
  await page.getByRole('button', { name: 'Discard and continue' }).click();
  await stroke(page);
  const before = await download(page, info, 'guard-original.nsketch');
  await page.locator('#new').click();
  await expect(page.locator('#guard')).toBeVisible();
  await page.keyboard.press('Escape');
  await expect(page.locator('#guard')).not.toBeVisible();
  expect((await download(page, info, 'guard-escape.nsketch')).bytes).toEqual(before.bytes);
  await page.locator('#open').click();
  await page.locator('#guard').getByRole('button', { name: 'Cancel', exact: true }).click();
  await expect(page.locator('#guard')).not.toBeVisible();
  expect((await download(page, info, 'guard-cancel.nsketch')).bytes).toEqual(before.bytes);
  await expect(page.locator('#status')).toContainText('Unsaved');
});
