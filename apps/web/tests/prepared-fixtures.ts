import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import type { Engine } from '../src/engine.ts';
import type { Surface } from 'canvaskit-wasm';
import { initializeSkia } from '../src/skia-module.ts';
export const kit = await initializeSkia({
  wasmBinary: await readFile('node_modules/canvaskit-wasm/bin/full/canvaskit.wasm'),
});
export function line(engine: Engine, points: number[][], width = 12) {
  engine.action('pen');
  engine.brush([1, 0, 0, 0.5], width);
  engine.begin(19n, 2);
  engine.append(
    19n,
    points.map(([x, y], i) => ({ x, y, time: i, id: BigInt(i + 1) })),
  );
  engine.commit(19n);
}
export function pixels(surface: Surface) {
  const image = surface.makeImageSnapshot();
  try {
    const bytes = image.readPixels(0, 0, {
      width: surface.width(),
      height: surface.height(),
      colorType: kit.ColorType.RGBA_8888,
      alphaType: kit.AlphaType.Unpremul,
      colorSpace: kit.ColorSpace.SRGB,
    });
    assert.ok(bytes);
    return Uint8Array.from(bytes);
  } finally {
    image.delete();
  }
}
export const camera = { x: 0, y: 0, zoom: 1, dpr: 1 };
