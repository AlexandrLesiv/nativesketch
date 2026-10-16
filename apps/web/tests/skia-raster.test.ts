import test from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { initializeSkia } from '../src/skia-module.ts';
import { setup, draw } from './fixtures.ts';
import { Scene } from '../src/scene.ts';
import { SkiaPainter } from '../src/skia-painter.ts';
import { SkiaPaths } from '../src/skia-paths.ts';
const options = {
  wasmBinary: await readFile('node_modules/canvaskit-wasm/bin/full/canvaskit.wasm'),
};
const kit = await initializeSkia(options);
function raster() {
  const surface = kit.MakeSurface(240, 240);
  assert.ok(surface);
  return surface;
}
function pixels(surface: NonNullable<ReturnType<typeof kit.MakeSurface>>) {
  const image = surface.makeImageSnapshot();
  try {
    const data = image.readPixels(0, 0, {
      width: 240,
      height: 240,
      colorType: kit.ColorType.RGBA_8888,
      alphaType: kit.AlphaType.Unpremul,
      colorSpace: kit.ColorSpace.SRGB,
    });
    assert.ok(data);
    return Uint8Array.from(data);
  } finally {
    image.delete();
  }
}
await test('actual CanvasKit raster draws actual portable scene and matches uncached complete paths', async () => {
  const { engine, work } = await setup();
  const scene = new Scene(engine),
    painter = new SkiaPainter(kit),
    surface = raster();
  try {
    draw(engine);
    const frame = scene.update([0, 0, 240, 240, 32]),
      camera = { x: 0, y: 0, zoom: 1, dpr: 1 };
    painter.draw(surface, frame, camera);
    const first = pixels(surface);
    const index = (50 * 240 + 50) * 4;
    assert.ok(first[index] < 245, 'known centerline is painted');
    assert.equal(first[(150 * 240 + 150) * 4], 255, 'translated empty region stays white');
    painter.draw(surface, frame, camera);
    assert.deepEqual(pixels(surface), first);
    const reference = new SkiaPainter(kit);
    try {
      reference.draw(surface, frame, camera);
      assert.deepEqual(pixels(surface), first);
    } finally {
      reference.dispose();
    }
    assert.equal(painter.paths.owned, 1);
    engine.action('undo');
    painter.draw(surface, scene.update([0, 0, 240, 240, 32]), camera);
    assert.equal(painter.paths.owned, 0);
  } finally {
    painter.dispose();
    surface.dispose();
    scene.dispose();
    await work.shutdown();
  }
});
await test('actual CanvasKit capacity rejection retains prior pixels and disposes bounded staged paths', async () => {
  const { engine, work } = await setup();
  const scene = new Scene(engine),
    paths = new SkiaPaths(kit, 32000000, 1);
  const painter = new SkiaPainter(kit, paths),
    surface = raster(),
    camera = { x: 0, y: 0, zoom: 1, dpr: 1 };
  try {
    draw(engine);
    painter.draw(surface, scene.update([0, 0, 240, 240, 32]), camera);
    const previous = pixels(surface);
    draw(engine, 5n, 100);
    assert.throws(
      () => painter.draw(surface, scene.update([0, 0, 240, 240, 32]), camera),
      /capacity/,
    );
    assert.deepEqual(pixels(surface), previous);
    assert.equal(paths.owned, 1);
  } finally {
    painter.dispose();
    assert.equal(paths.owned, 0);
    surface.dispose();
    scene.dispose();
    await work.shutdown();
  }
});
