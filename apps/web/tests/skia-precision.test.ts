import test from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { initializeSkia } from '../src/skia-module.ts';
import { SkiaPainter } from '../src/skia-painter.ts';
import { Scene } from '../src/scene.ts';
import { setup, draw } from './fixtures.ts';
const kit = await initializeSkia({
  wasmBinary: await readFile('node_modules/canvaskit-wasm/bin/full/canvaskit.wasm'),
});
await test('actual CanvasKit preserves small portable geometry at signed billion-unit origins with fractional transforms', async () => {
  const outputs: Uint8Array[] = [];
  for (const origin of [0, 1e9 - 1000, -1e9 + 1000]) {
    const { engine, work } = await setup(),
      scene = new Scene(engine),
      painter = new SkiaPainter(kit);
    const surface = kit.MakeSurface(240, 240);
    assert.ok(surface);
    try {
      draw(engine, 99n, origin);
      engine.action('select');
      engine.select(origin + 50, 50, 10);
      engine.beginTransform();
      engine.transform(1.25, 0, 0);
      engine.commitTransform();
      const camera = { x: origin * 1.25, y: 0, zoom: 0.875, dpr: 1.5 };
      const frame = scene.update([
        camera.x,
        0,
        240 / (camera.zoom * camera.dpr),
        240 / (camera.zoom * camera.dpr),
        32,
      ]);
      painter.draw(surface, frame, camera);
      const image = surface.makeImageSnapshot();
      try {
        const bytes = image.readPixels(0, 0, {
          width: 240,
          height: 240,
          colorType: kit.ColorType.RGBA_8888,
          alphaType: kit.AlphaType.Unpremul,
          colorSpace: kit.ColorSpace.SRGB,
        });
        assert.ok(bytes);
        outputs.push(Uint8Array.from(bytes));
      } finally {
        image.delete();
      }
    } finally {
      painter.dispose();
      surface.dispose();
      scene.dispose();
      await work.shutdown();
    }
  }
  assert.deepEqual(outputs[1], outputs[0]);
  assert.deepEqual(outputs[2], outputs[0]);
});
await test('actual CanvasKit fills complete paged translucent portable geometry once and suppresses preview originals', async () => {
  const { engine, work } = await setup(),
    scene = new Scene(engine),
    painter = new SkiaPainter(kit);
  const surface = kit.MakeSurface(400, 240);
  assert.ok(surface);
  try {
    engine.brush([1, 0, 0, 0.5], 12);
    engine.begin(1n, 2);
    engine.append(
      1n,
      Array.from({ length: 1000 }, (_, i) => ({
        x: 20 + i / 3,
        y: 100,
        time: i,
        id: BigInt(i + 1),
      })),
    );
    engine.commit(1n);
    let frame = scene.update([0, 0, 400, 240, 32]);
    assert.ok([...frame.geometry.values()][0].commands.length / 88 > 744);
    engine.action('select');
    engine.select(100, 100, 10);
    engine.beginTransform();
    engine.transform(2, 0, 0);
    frame = scene.update([0, 0, 400, 240, 32]);
    assert.equal(frame.strokes.length, 1);
    painter.draw(surface, frame, { x: 0, y: 0, zoom: 1, dpr: 1 });
    const image = surface.makeImageSnapshot();
    try {
      const bytes = image.readPixels(0, 0, {
        width: 400,
        height: 240,
        colorType: kit.ColorType.RGBA_8888,
        alphaType: kit.AlphaType.Unpremul,
        colorSpace: kit.ColorSpace.SRGB,
      });
      assert.ok(bytes);
      assert.equal(bytes[(100 * 400 + 100) * 4 + 1], 255, 'suppressed original remains white');
      const green = bytes[(200 * 400 + 100) * 4 + 1];
      assert.ok(green >= 126 && green <= 128, `single alpha composite: ${green}`);
    } finally {
      image.delete();
    }
  } finally {
    painter.dispose();
    surface.dispose();
    scene.dispose();
    await work.shutdown();
  }
});
await test('actual portable huge-span scene renders shared prepared pixels across the viewport', async () => {
  const { engine, work } = await setup(),
    scene = new Scene(engine),
    painter = new SkiaPainter(kit);
  const surface = kit.MakeSurface(240, 240);
  assert.ok(surface);
  try {
    draw(engine);
    painter.draw(surface, scene.update([0, 0, 240, 240, 32]), { x: 0, y: 0, zoom: 1, dpr: 1 });
    const before = surface.makeImageSnapshot(),
      previous = before.encodeToBytes();
    before.delete();
    engine.begin(100n, 2);
    engine.append(100n, [
      { x: -1e8, y: 100, time: 0, id: 1n },
      { x: 1e8, y: 100, time: 1, id: 2n },
    ]);
    engine.commit(100n);
    const frame = scene.update([0, 0, 240, 240, 32]);
    assert.equal(frame.strokes.length, 2);
    painter.draw(surface, frame, { x: 0, y: 0, zoom: 1, dpr: 1 });
    const after = surface.makeImageSnapshot();
    try {
      assert.notDeepEqual(after.encodeToBytes(), previous);
      const data = after.readPixels(0, 0, {
        width: 240,
        height: 240,
        colorType: kit.ColorType.RGBA_8888,
        alphaType: kit.AlphaType.Unpremul,
        colorSpace: kit.ColorSpace.SRGB,
      });
      assert.ok(data);
      assert.ok(data[(100 * 240 + 120) * 4] < 245);
      assert.equal(data[(150 * 240 + 120) * 4], 255);
    } finally {
      after.delete();
    }
    assert.equal(engine.state().count, 2n);
  } finally {
    painter.dispose();
    surface.dispose();
    scene.dispose();
    await work.shutdown();
  }
});
