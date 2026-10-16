import test from 'node:test';
import assert from 'node:assert/strict';
import { setup, draw } from './fixtures.ts';
import { Scene } from '../src/scene.ts';
await test('portable Wasm scene pages beyond descriptor and geometry caps remain complete', async () => {
  const { engine: e, work } = await setup();
  for (let i = 0; i < 1366; i++) draw(e, BigInt(i + 1), i / 10);
  const scene = new Scene(e),
    frame = scene.update([0, 0, 500, 500, 32]);
  assert.equal(frame.strokes.length, 1366);
  assert.equal(frame.geometry.size, 1366);
  e.begin(9999n, 2);
  e.append(
    9999n,
    Array.from({ length: 1000 }, (_, i) => ({
      x: i / 3,
      y: 200 + Math.sin(i / 10) * 50,
      time: i,
      id: BigInt(i + 1),
    })),
  );
  e.commit(9999n);
  const detailed = scene.update([0, 0, 500, 500, 32]);
  const last = detailed.geometry.get(detailed.strokes.at(-1)!.geometry)!;
  assert.ok(last.commands.length / 88 > 744);
  assert.equal(detailed.strokes.length, 1367);
  e.action('undo');
  assert.equal(scene.update([0, 0, 500, 500, 32]).geometry.size, 1366);
  scene.dispose();
  await work.shutdown();
});
await test('portable Wasm bounded geometry admission retains previous coherent frame at capacity', async () => {
  const { engine: e, work } = await setup();
  const scene = new Scene(e, 100),
    viewport = [0, 0, 500, 500, 32];
  const empty = scene.update(viewport);
  draw(e);
  assert.throws(() => scene.update(viewport), /capacity/);
  assert.equal(scene.frame, empty);
  e.action('undo');
  assert.equal(scene.update(viewport).strokes.length, 0);
  scene.dispose();
  await work.shutdown();
});
await test('portable Wasm live scene generations reuse unchanged committed root descriptors', async () => {
  const { engine: e, work } = await setup();
  draw(e);
  const scene = new Scene(e),
    viewport = [0, 0, 500, 500, 32];
  let pages = 0;
  const original = e.transport.call.bind(e.transport);
  e.transport.call = (op, body) => {
    if (op === 41) pages++;
    return original(op, body);
  };
  scene.update(viewport);
  assert.equal(pages, 1);
  e.begin(7n, 2);
  for (let i = 0; i < 4; i++) {
    e.append(7n, [{ x: 200 + i, y: 100, time: i, id: BigInt(i + 1) }]);
    scene.update(viewport);
    assert.equal(pages, 1);
  }
  e.commit(7n);
  scene.update(viewport);
  assert.equal(pages, 2);
  scene.dispose();
  await work.shutdown();
});
