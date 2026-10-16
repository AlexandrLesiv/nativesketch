import { test, expect } from '@playwright/test';
import { ready, stroke, download, inkPixels } from './helpers.ts';
test('WEB-HISTORY-001 mouse ink selection scale delete and one-operation history', async ({
  page,
}, info) => {
  await ready(page);
  await stroke(page);
  await expect(page.locator('#status')).toContainText('1 stroke');
  expect(await inkPixels(page)).toBeGreaterThan(100);
  const original = await download(page, info, 'original.nsketch');
  expect(original.bytes.readBigUInt64LE(8)).toBe(1n);
  await page.locator('#undo').click();
  await expect(page.locator('#status')).toContainText('0 strokes');
  expect(await inkPixels(page)).toBe(0);
  await page.locator('#redo').click();
  expect((await download(page, info, 'redo.nsketch')).bytes).toEqual(original.bytes);
  await page.locator('#next').click();
  await expect(page.locator('#select')).toHaveAttribute('aria-pressed', 'true');
  await page.locator('#scale').fill('150');
  await page.locator('#apply').click();
  const scaled = await download(page, info, 'scaled.nsketch');
  expect(scaled.bytes).not.toEqual(original.bytes);
  await page.locator('#undo').click();
  expect((await download(page, info, 'unscaled.nsketch')).bytes).toEqual(original.bytes);
  await page.locator('#redo').click();
  expect((await download(page, info, 'rescaled.nsketch')).bytes).toEqual(scaled.bytes);
  await page.locator('#delete').click();
  await expect(page.locator('#status')).toContainText('0 strokes');
  await page.locator('#canvas').focus();
  await page.keyboard.press('ControlOrMeta+z');
  await expect(page.locator('#status')).toContainText('1 stroke');
});
test('WEB-FOCUS-001 native number-field editing does not change drawing commands', async ({
  page,
}, info) => {
  await ready(page);
  await stroke(page);
  await page.locator('#next').click();
  const before = await download(page, info, 'before-fields.nsketch');
  for (const id of ['width', 'scale']) {
    await page.locator(`#${id}`).focus();
    await page.keyboard.press('ControlOrMeta+a');
    await page.keyboard.type('25');
    await page.keyboard.press('ControlOrMeta+z');
    await page.keyboard.press('Backspace');
    await expect(page.locator(`#${id}`)).toBeFocused();
    await expect(page.locator('#status')).toContainText('1 stroke');
    await expect(page.locator('#select')).toHaveAttribute('aria-pressed', 'true');
  }
  expect((await download(page, info, 'after-fields.nsketch')).bytes).toEqual(before.bytes);
});
test('WEB-CANCEL-001 Escape and viewport resize cancel held drafts', async ({ page }, info) => {
  await ready(page);
  await stroke(page);
  const before = await download(page, info, 'committed.nsketch');
  for (const cause of ['escape', 'resize']) {
    const box = await page.locator('#canvas').boundingBox();
    if (!box) throw new Error('Canvas missing');
    await page.mouse.move(box.x + 300, box.y + 100);
    await page.mouse.down();
    await page.mouse.move(box.x + 390, box.y + 150, { steps: 4 });
    if (cause === 'escape') await page.keyboard.press('Escape');
    else {
      await page.setViewportSize({ width: 1000, height: 700 });
      // CDP viewport completion precedes ResizeObserver delivery. Observe the real
      // backing-store resize before releasing; never inject the app's callback.
      await expect
        .poll(() =>
          page.locator('#canvas').evaluate((canvas) => {
            const element = canvas as HTMLCanvasElement;
            return (
              element.width === Math.round(element.getBoundingClientRect().width * devicePixelRatio)
            );
          }),
        )
        .toBe(true);
    }
    await page.mouse.up();
    expect((await download(page, info, `${cause}.nsketch`)).bytes).toEqual(before.bytes);
    await expect(page.locator('#status')).toContainText('1 stroke');
  }
});
