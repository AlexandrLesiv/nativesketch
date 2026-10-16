import { test, expect } from '@playwright/test';
import { ready, stroke, inkPixels, download } from './helpers.ts';
import type { Page } from '@playwright/test';
async function contextEvent(page: Page, action: 'loseContext' | 'restoreContext') {
  await page.locator('#canvas').evaluate((canvas: HTMLCanvasElement, method) => {
    const gl = canvas.getContext('webgl2') ?? canvas.getContext('webgl');
    const state = window as Window & { sketchLoss?: WEBGL_lose_context };
    const extension =
      method === 'loseContext' ? gl?.getExtension('WEBGL_lose_context') : state.sketchLoss;
    if (extension) state.sketchLoss = extension;
    if (!extension) throw Error('Software context-loss extension unavailable');
    extension[method]();
  }, action);
}
test('WEB-SURFACE-001 injected initial GL refusal uses real Skia raster and preserves one-operation input', async ({
  page,
}, info) => {
  await page.addInitScript(() => {
    const original: unknown = Object.getOwnPropertyDescriptor(
      HTMLCanvasElement.prototype,
      'getContext',
    )?.value;
    if (typeof original !== 'function') throw Error('Canvas context factory unavailable');
    const originalCanvases = new WeakSet<HTMLCanvasElement>();
    let captured = false;
    Object.defineProperty(window, 'sketchOriginalCanvas', {
      value: (canvas: HTMLCanvasElement) => originalCanvases.has(canvas),
    });
    Object.defineProperty(HTMLCanvasElement.prototype, 'getContext', {
      value: function (this: HTMLCanvasElement, kind: string, ...args: unknown[]) {
        if (kind === 'webgl' || kind === 'webgl2' || kind === 'experimental-webgl') {
          if (!captured) originalCanvases.add(this);
          captured = true;
          return null;
        }
        return Reflect.apply(original, this, [kind, ...args]);
      },
    });
  });
  await ready(page);
  await expect(page.locator('#canvas')).toHaveAttribute('data-renderer', 'CanvasKit 0.42.0 raster');
  const originalRetained = () =>
    page
      .locator('#canvas')
      .evaluate((canvas) =>
        (
          window as Window & { sketchOriginalCanvas?: (element: Element) => boolean }
        ).sketchOriginalCanvas?.(canvas),
      );
  expect(await originalRetained()).toBe(true);
  await stroke(page);
  expect(await inkPixels(page)).toBeGreaterThan(100);
  await page.setViewportSize({ width: 1000, height: 700 });
  expect(await originalRetained()).toBe(true);
  await page.locator('#width').focus();
  await expect(page.locator('#width')).toBeFocused();
  await page.locator('#undo').click();
  await expect(page.locator('#status')).toContainText('0 strokes');
  await page.locator('#redo').click();
  await expect(page.locator('#status')).toContainText('1 stroke');
  await info.attach('raster-ui', { body: await page.screenshot(), contentType: 'image/png' });
  await info.attach('raster-backend', {
    body: 'Actual CanvasKit CPU raster; injected WebGL refusal',
    contentType: 'text/plain',
  });
});
test('WEB-CONTEXT-001 software WebGL loss cancels draft and restores latest retained scene without stealing focus', async ({
  page,
}, info) => {
  await ready(page);
  await expect(page.locator('#canvas')).toHaveAttribute('data-renderer', 'CanvasKit 0.42.0 webgl');
  const renderer = await page.locator('#canvas').evaluate((canvas: HTMLCanvasElement) => {
    const gl = canvas.getContext('webgl2') ?? canvas.getContext('webgl');
    const ext = gl?.getExtension('WEBGL_debug_renderer_info');
    return ext ? String(gl!.getParameter(ext.UNMASKED_RENDERER_WEBGL)) : 'unavailable';
  });
  expect(renderer).toContain('SwiftShader');
  await info.attach('actual-gl-renderer', { body: renderer, contentType: 'text/plain' });
  await stroke(page);
  const box = (await page.locator('#canvas').boundingBox())!;
  await page.mouse.move(box.x + 100, box.y + 220);
  await page.mouse.down();
  await page.mouse.move(box.x + 180, box.y + 240);
  await contextEvent(page, 'loseContext');
  await expect(page.locator('#canvas')).toHaveAttribute('aria-disabled', 'true');
  await page.mouse.up();
  await page.locator('#width').focus();
  await contextEvent(page, 'restoreContext');
  await expect(page.locator('#canvas')).toHaveAttribute('aria-disabled', 'false');
  await expect(page.locator('#width')).toBeFocused();
  await expect(page.locator('#status')).toContainText('1 stroke');
  await info.attach('restored-webgl-ui', {
    body: await page.screenshot(),
    contentType: 'image/png',
  });
  expect(await inkPixels(page)).toBeGreaterThan(100);
  await stroke(page, 100);
  await expect(page.locator('#status')).toContainText('2 strokes');
  await page.locator('#undo').click();
  await expect(page.locator('#status')).toContainText('1 stroke');
  await page.locator('#undo').click();
  await expect(page.locator('#status')).toContainText('0 strokes');
});
test('WEB-RENDER-ERROR-001 injected restoration failure retains downloadable document', async ({
  page,
}, info) => {
  await ready(page);
  await stroke(page);
  const original = await download(page, info, 'before-renderer-failure.nsketch');
  await contextEvent(page, 'loseContext');
  await expect(page.locator('#canvas')).toHaveAttribute('aria-disabled', 'true');
  await page.locator('#canvas').evaluate((canvas: HTMLCanvasElement) => {
    Object.defineProperty(canvas, 'width', { get: () => 20000 });
    Object.defineProperty(canvas, 'height', { get: () => 20000 });
  });
  await contextEvent(page, 'restoreContext');
  await expect(page.locator('#status')).toContainText('Rendering unavailable');
  await expect(page.locator('#canvas')).toHaveAttribute('data-renderer-error', /pixel budget/);
  const result = await download(page, info, 'retained-renderer-failure.nsketch');
  expect(result.bytes).toEqual(original.bytes);
  await info.attach('renderer-failure-ui', {
    body: await page.screenshot(),
    contentType: 'image/png',
  });
});
