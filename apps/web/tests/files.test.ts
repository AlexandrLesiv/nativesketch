import test from 'node:test';
import assert from 'node:assert/strict';
import { setup, draw, exported, open, settle } from './fixtures.ts';
await test('portable Wasm issued exports preserve dirty and clean reopened state', async () => {
  const { engine: e, work: w } = await setup();
  draw(e);
  const bytes = await exported(e, w);
  assert.equal(e.state().dirty, true);
  open(e, w, { size: bytes.length, arrayBuffer: async () => bytes.slice().buffer });
  await settle(w);
  assert.equal(e.state().dirty, false);
  assert.equal(e.state().count, 1n);
  assert.deepEqual(await exported(e, w), bytes);
  assert.equal(e.state().dirty, false);
  await w.shutdown();
});
await test('portable Wasm delayed reads retire after intervening draft without overwriting input', async () => {
  const { engine: e, work: w } = await setup();
  draw(e);
  const bytes = await exported(e, w);
  let resolve!: (b: ArrayBuffer) => void;
  open(e, w, {
    size: bytes.length,
    arrayBuffer: () =>
      new Promise((r) => {
        resolve = r;
      }),
  });
  const pending = settle(w);
  e.begin(7n, 2);
  e.append(7n, [{ x: 50, y: 50, time: 0, id: 1n }]);
  resolve(bytes.slice().buffer);
  await pending;
  assert.equal(e.state().interaction, 1);
  assert.equal(e.state().count, 1n);
  e.commit(7n);
  assert.equal(e.state().count, 2n);
  await w.shutdown();
});
await test('portable Wasm corrupt and oversized read failures retain the document', async () => {
  const { engine: e, work: w } = await setup();
  draw(e);
  for (const size of [3, 67108865]) {
    open(e, w, { size, arrayBuffer: async () => new Uint8Array([1, 2, 3]).buffer });
    const effects = await settle(w);
    assert.equal(new DataView(effects[0].buffer).getUint32(232, true), 9);
    assert.equal(e.state().count, 1n);
  }
  await w.shutdown();
});
await test('portable Wasm rejected New preserves pending read; explicit cancellation retires after later New', async () => {
  const { engine: e, work: w } = await setup();
  draw(e);
  const bytes = await exported(e, w);
  let release!: (b: ArrayBuffer) => void;
  const queued = open(e, w, {
    size: bytes.length,
    arrayBuffer: () =>
      new Promise((r) => {
        release = r;
      }),
  });
  const delayed = settle(w);
  const { u32, u64, words } = await import('../src/wire.ts');
  assert.throws(() => e.file(4));
  assert.equal(e.state().count, 1n);
  e.transport.call(33, words(u64(queued, 248)));
  let navigation = e.file(4);
  if (u32(navigation, 232) === 3) navigation = e.reply(u64(navigation, 240), 3);
  await settle(w);
  assert.equal(e.state().count, 0n);
  release(bytes.slice().buffer);
  await delayed;
  assert.equal(e.state().count, 0n);
  await w.shutdown();
});
