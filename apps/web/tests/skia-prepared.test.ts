import test from 'node:test';
import assert from 'node:assert/strict';
import { setup } from './fixtures.ts';
import { Scene } from '../src/scene.ts';
import { SkiaPainter } from '../src/skia-painter.ts';
import { kit, line, pixels, camera } from './prepared-fixtures.ts';
await test('actual shared prepared signed crossing paths match local references with asymmetric fractional backing and one alpha fill', async () => {
  const outputs: Uint8Array[] = [];
  for (const extent of [1000, 999999000, -999999000]) {
    const { engine, work } = await setup(),
      scene = new Scene(engine),
      painter = new SkiaPainter(kit);
    const surface = kit.MakeSurface(133, 219);
    assert.ok(surface);
    try {
      line(engine, [
        [-extent, 40],
        [extent, 40],
      ]);
      painter.draw(surface, scene.update([-10.25, 0.5, 100.5, 160.25, 2]), camera);
      const data = pixels(surface);
      outputs.push(data);
      assert.ok(data[(54 * 133 + 66) * 4 + 1] >= 126 && data[(54 * 133 + 66) * 4 + 1] <= 128);
      for (let x = 0; x < 133; x++)
        assert.equal(data[(54 * 133 + x) * 4 + 1], 128, 'no clipping holes');
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
await test('actual prepared multi-page preview paths cache only matching viewport transform and dimensions', async () => {
  const { engine, work } = await setup(),
    scene = new Scene(engine),
    painter = new SkiaPainter(kit);
  const surface = kit.MakeSurface(133, 137);
  assert.ok(surface);
  let prepares = 0,
    pages = 0,
    releases = 0;
  const original = engine.transport.call.bind(engine.transport);
  engine.transport.call = (op, body) => {
    if (op === 45) prepares++;
    if (op === 46) pages++;
    if (op === 47) releases++;
    return original(op, body);
  };
  try {
    line(
      engine,
      Array.from({ length: 300 }, (_, i) => [i % 2 ? 999999010 : 0, 20]),
    );
    let frame = scene.update([999999000, 0, 100, 100, 2]);
    painter.draw(surface, frame, camera);
    assert.ok(pages > 1);
    assert.equal(releases, 1);
    const before = pixels(surface),
      first = prepares;
    painter.draw(surface, frame, camera);
    assert.equal(prepares, first);
    assert.deepEqual(pixels(surface), before);
    const reference = new SkiaPainter(kit);
    try {
      reference.draw(surface, frame, camera);
      assert.deepEqual(pixels(surface), before);
    } finally {
      reference.dispose();
    }
    frame = scene.update([999998990, 0, 100, 100, 2]);
    const changed = prepares;
    painter.draw(surface, frame, camera);
    assert.equal(prepares, changed + 1);
    assert.notDeepEqual(pixels(surface), before);
    const resized = kit.MakeSurface(134, 139);
    assert.ok(resized);
    try {
      const n = prepares;
      painter.draw(resized, frame, camera);
      assert.equal(prepares, n + 1);
    } finally {
      resized.dispose();
    }
    engine.action('next');
    engine.beginTransform();
    engine.transform(2, 999999010, 20);
    frame = scene.update([999999000, 0, 100, 100, 2]);
    assert.equal(frame.strokes.length, 1);
    assert.equal(frame.strokes[0].selector, 2);
    painter.draw(surface, frame, camera);
    assert.equal(painter.paths.owned, 1);
    const data = pixels(surface);
    assert.equal(data[(27 * 133 + 5) * 4 + 1], 128, 'preview is a single fill');
    assert.equal(releases, prepares);
  } finally {
    painter.dispose();
    surface.dispose();
    scene.dispose();
    await work.shutdown();
  }
});
await test('actual prepared enclosing and grazing giant curves preserve filled interiors', async () => {
  for (const x of [0, 5999995]) {
    const { engine, work } = await setup(),
      scene = new Scene(engine),
      painter = new SkiaPainter(kit);
    const surface = kit.MakeSurface(133, 137);
    assert.ok(surface);
    try {
      line(engine, [[0, 0]]);
      engine.action('next');
      engine.beginTransform();
      engine.transform(1e6, 0, 0);
      engine.commitTransform();
      painter.draw(surface, scene.update([x, -50, 100, 100, 2]), camera);
      const data = pixels(surface);
      assert.equal(
        data[(68 * 133 + 1) * 4 + 1],
        128,
        'enclosed or grazing interior remains filled',
      );
      if (x) assert.equal(data[(68 * 133 + 132) * 4 + 1], 255, 'outside giant curve stays white');
      else
        for (let i = 1; i < data.length; i += 4)
          assert.equal(data[i], 128, 'enclosing contour has no holes');
    } finally {
      painter.dispose();
      surface.dispose();
      scene.dispose();
      await work.shutdown();
    }
  }
});
await test('actual prepared empty pixel path releases its lease and paints no false coverage', async () => {
  const { engine, work } = await setup(),
    scene = new Scene(engine),
    painter = new SkiaPainter(kit);
  const surface = kit.MakeSurface(133, 137);
  assert.ok(surface);
  try {
    line(engine, [
      [-1e8, -1e8],
      [1e8, 1e8],
    ]);
    const frame = scene.update([0, 1000, 100, 100, 2]);
    assert.equal(frame.strokes.length, 1);
    painter.draw(surface, frame, camera);
    for (const value of pixels(surface)) assert.equal(value, 255);
    const prepared = frame.prepare!(frame.strokes[0], 133, 137, 0);
    assert.ok(prepared);
    assert.equal(prepared.commands.length, 0);
  } finally {
    painter.dispose();
    surface.dispose();
    scene.dispose();
    await work.shutdown();
  }
});
