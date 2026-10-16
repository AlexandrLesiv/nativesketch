import test from 'node:test';
import assert from 'node:assert/strict';
import { setup } from './fixtures.ts';
import { Scene } from '../src/scene.ts';
import { SkiaPainter } from '../src/skia-painter.ts';
import { words, integers, join, u64 } from '../src/wire.ts';
import { kit, line, pixels, camera } from './prepared-fixtures.ts';
await test('actual prepared consumer rejects stale scenes and wrong provenance without retaining a wire lease', async () => {
  const { engine, work } = await setup(),
    scene = new Scene(engine);
  try {
    line(engine, [
      [-1e8, 40],
      [1e8, 40],
    ]);
    const old = scene.update([0, 0, 100, 100, 2]);
    assert.ok(old.prepare);
    assert.throws(
      () =>
        old.prepare!(
          { ...old.strokes[0], geometry: old.strokes[0].geometry + 1n },
          133,
          137,
          100000,
        ),
      /provenance/,
    );
    assert.ok(old.prepare(old.strokes[0], 133, 137, 100000));
    const current = scene.update([1, 0, 100, 100, 2]);
    assert.throws(() => old.prepare!(old.strokes[0], 133, 137, 100000));
    assert.ok(current.prepare!(current.strokes[0], 133, 137, 100000));
  } finally {
    scene.dispose();
    await work.shutdown();
  }
});
await test('injected rejected and malformed prepared pages release leases and retain prior pixels and path owners', async () => {
  const { engine, work } = await setup(),
    scene = new Scene(engine),
    painter = new SkiaPainter(kit);
  const surface = kit.MakeSurface(133, 137);
  assert.ok(surface);
  const original = engine.transport.call.bind(engine.transport);
  let reject: false | true | 'malformed' = false,
    releases = 0;
  engine.transport.call = (op, body) => {
    if (op === 46 && reject === true) throw Error('injected prepared page rejection');
    if (op === 46 && reject === 'malformed') return original(op, body).subarray(0, 80);
    if (op === 47) releases++;
    return original(op, body);
  };
  try {
    line(engine, [
      [10, 20],
      [30, 20],
    ]);
    painter.draw(surface, scene.update([0, 0, 100, 100, 2]), camera);
    const previous = pixels(surface),
      owned = painter.paths.owned;
    line(engine, [
      [-1e8, 40],
      [1e8, 40],
    ]);
    reject = true;
    const frame = scene.update([0, 0, 100, 100, 2]);
    assert.throws(() => painter.draw(surface, frame, camera), /injected prepared page/);
    assert.deepEqual(pixels(surface), previous);
    assert.equal(painter.paths.owned, owned);
    assert.equal(releases, 1);
    reject = 'malformed';
    assert.throws(() => painter.draw(surface, frame, camera), /Incomplete prepared path/);
    assert.deepEqual(pixels(surface), previous);
    assert.equal(painter.paths.owned, owned);
    assert.equal(releases, 2);
    reject = false;
    painter.draw(surface, frame, camera);
    assert.equal(releases, 3);
    assert.notDeepEqual(pixels(surface), previous);
  } finally {
    reject = false;
    painter.dispose();
    surface.dispose();
    scene.dispose();
    await work.shutdown();
  }
});
await test('injected prepared release rejection preserves ownership until retry and explicit consumer disposal', async () => {
  const { engine, work } = await setup(),
    scene = new Scene(engine);
  const original = engine.transport.call.bind(engine.transport);
  let reject = true,
    held = 0n;
  engine.transport.call = (op, body) => {
    if (op === 47 && reject) throw Error('injected prepared release rejection');
    const reply = original(op, body);
    if (op === 45) held = u64(reply, 80);
    return reply;
  };
  try {
    line(engine, [
      [-1e8, 40],
      [1e8, 40],
    ]);
    const frame = scene.update([0, 0, 100, 100, 2]);
    assert.throws(() => frame.prepare!(frame.strokes[0], 133, 137, 100000), /release rejection/);
    assert.notEqual(held, 0n);
    assert.ok(original(46, join(words(held), integers(0, 744))).length > 80);
    reject = false;
    scene.dispose();
    assert.throws(() => original(46, join(words(held), integers(0, 744))));
  } finally {
    reject = false;
    scene.dispose();
    await work.shutdown();
  }
});
await test('actual prepared allocation budget rejection retires output and leaves the scene usable', async () => {
  const { engine, work } = await setup(),
    scene = new Scene(engine);
  try {
    line(engine, [
      [-1e8, 40],
      [1e8, 40],
    ]);
    const frame = scene.update([0, 0, 100, 100, 2]);
    assert.throws(() => frame.prepare!(frame.strokes[0], 133, 137, 0), /capacity/);
    assert.ok(frame.prepare!(frame.strokes[0], 133, 137, 100000));
  } finally {
    scene.dispose();
    await work.shutdown();
  }
});
