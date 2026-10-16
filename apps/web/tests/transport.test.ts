import test from 'node:test';
import assert from 'node:assert/strict';
import { setup, draw } from './fixtures.ts';
import { words, u64 } from '../src/wire.ts';
await test('portable Wasm memory growth reacquires views and output exhaustion preserves owner', async () => {
  const { engine: e, work } = await setup(),
    t = e.transport;
  draw(e);
  const previous = t.memory.buffer;
  t.memory.grow(1);
  assert.notEqual(t.memory.buffer, previous);
  assert.equal(e.state().count, 1n);
  const replies: number[] = [];
  assert.throws(() => {
    for (;;)
      replies.push(
        t.using(t.message(3, words(e.session)), (p) => t.invoke('app_call', [t.app, p]).reply),
      );
  });
  assert.ok(replies.length > 0);
  for (const r of replies) t.invoke('buffer_free', [r]);
  assert.equal(e.state().count, 1n);
  await work.shutdown();
});
await test('portable Wasm rejected work transfer retains ownership for valid completion', async () => {
  const { engine: e, work } = await setup(),
    t = e.transport;
  work.supply(u64(e.file(3), 240));
  const w = t.take();
  const operation = u64(t.work(w, 100).bytes, 72);
  assert.throws(() => t.work(w, 102));
  const encoded = t.work(w, 101);
  assert.equal(encoded.result, 0);
  const result = t.work(w, 104, new Uint8Array([4, 0, 0, 0, 0, 0, 0, 0]));
  const foreign = await setup();
  assert.throws(() => foreign.engine.transport.complete(result.result));
  const info = e.projection();
  assert.ok(u64(info, 72));
  t.complete(result.result);
  t.call(32, words(e.session, operation));
  work.releases();
  await foreign.work.shutdown();
  await work.shutdown();
});
