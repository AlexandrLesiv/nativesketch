import { expect } from '@playwright/test';
import type { Page, TestInfo } from '@playwright/test';
import { readFile } from 'node:fs/promises';
export async function ready(page: Page) {
  await page.goto('./');
  await expect(page.locator('#status')).toContainText('0 strokes');
}
export async function stroke(page: Page, offset = 0) {
  const box = await page.locator('#canvas').boundingBox();
  if (!box) throw new Error('Canvas not visible');
  await page.mouse.move(box.x + 100, box.y + 100 + offset);
  await page.mouse.down();
  await page.mouse.move(box.x + 260, box.y + 160 + offset, { steps: 8 });
  await page.mouse.up();
}
export async function download(page: Page, info: TestInfo, name: string) {
  const waiting = page.waitForEvent('download');
  await page.locator('#save').click();
  const result = await waiting;
  expect(result.suggestedFilename()).toBe('Drawing.nsketch');
  const path = info.outputPath(name);
  await result.saveAs(path);
  const bytes = await readFile(path);
  expect(bytes.subarray(0, 8).toString()).toBe('NSKETCH1');
  await info.attach(name, { path, contentType: 'application/octet-stream' });
  return { bytes, path };
}
export async function inkPixels(page: Page) {
  await page.evaluate(() => new Promise<void>((resolve) => requestAnimationFrame(() => resolve())));
  return page.locator('#canvas').evaluate((canvas: HTMLCanvasElement) => {
    const data = canvas.getContext('2d')!.getImageData(0, 0, canvas.width, canvas.height).data;
    let count = 0;
    for (let i = 0; i < data.length; i += 4)
      if (data[i] < 245 || data[i + 1] < 245 || data[i + 2] < 245) count++;
    return count;
  });
}
