import { test, expect } from '@playwright/test';
import { ready, stroke, download } from './helpers.ts';
test('WEB-BRUSH-002 default width color and integer width persist in saved drawing', async ({
  page,
}, info) => {
  await ready(page);
  await page.locator('#color').fill('#ff0000');
  await page.locator('#color').dispatchEvent('change');
  await stroke(page);
  const red = await download(page, info, 'red-width4.nsketch');
  await page.locator('#width').fill('12');
  await page.locator('#color').fill('#00ff00');
  await page.locator('#color').dispatchEvent('change');
  await stroke(page, 80);
  const changed = await download(page, info, 'green-width12.nsketch');
  expect(changed.bytes).not.toEqual(red.bytes);
  // NSKETCH1 records store RGBA then width after their stroke identity and sample count.
  expect(red.bytes.includes(Buffer.from(new Float64Array([1, 0, 0, 1, 4]).buffer))).toBe(true);
  expect(changed.bytes.includes(Buffer.from(new Float64Array([0, 1, 0, 1, 12]).buffer))).toBe(true);
  await expect(page.locator('#error')).not.toBeVisible();
  await page.screenshot({ path: info.outputPath('brush-controls.png') });
  await info.attach('brush-controls', {
    path: info.outputPath('brush-controls.png'),
    contentType: 'image/png',
  });
});
