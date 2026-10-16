import type { Page } from '@playwright/test';
/** Inspect the real composited screenshot, independent of the production rendering backend. */
export async function screenshotInk(page: Page) {
  const screenshot = await page.locator('#canvas').screenshot({ scale: 'css' });
  return page.evaluate(
    async (bytes) => {
      const bitmap = await createImageBitmap(
        new Blob([Uint8Array.from(bytes)], { type: 'image/png' }),
      );
      const canvas = new OffscreenCanvas(bitmap.width, bitmap.height);
      const context = canvas.getContext('2d')!;
      context.drawImage(bitmap, 0, 0);
      bitmap.close();
      const { data, width, height } = context.getImageData(0, 0, canvas.width, canvas.height);
      let count = 0,
        left = width,
        top = height,
        right = -1,
        bottom = -1;
      for (let y = 0; y < height; y++)
        for (let x = 0; x < width; x++) {
          const i = (y * width + x) * 4;
          if (data[i] < 245 || data[i + 1] < 245 || data[i + 2] < 245) {
            count++;
            left = Math.min(left, x);
            top = Math.min(top, y);
            right = Math.max(right, x);
            bottom = Math.max(bottom, y);
          }
        }
      return { count, left, top, right, bottom };
    },
    [...screenshot],
  );
}
