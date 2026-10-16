import assert from 'node:assert/strict';
import { packet, words, u32, u64, fixture } from './harness.ts';
import type { Harness } from './harness.ts';
import { context, file, queue, take, worker, complete, close } from './flow.ts';
export function checkPolicyBounds(h: Harness): void {
  const a = h.create();
  const s = u64(h.call(a, fixture('create.policy.1')), 72);
  const empty = h.call(a, packet(9, words(s)));
  assert.equal(empty.length, 128);
  assert.equal(u32(empty, 88), 0);
  assert.deepEqual(empty.slice(92), new Uint8Array(36));
  const document = u64(empty, 72);
  let b = new Uint8Array(48);
  b.set(context(h, a, s));
  let v = new DataView(b.buffer);
  v.setUint32(24, 6, true);
  v.setBigUint64(32, 7n, true);
  v.setUint32(40, 2, true);
  h.call(a, packet(10, b));
  b = fixture('sample.0.0').slice(24);
  b.set(context(h, a, s));
  v = new DataView(b.buffer);
  v.setFloat64(56, 900e6 + 0.125, true);
  v.setFloat64(64, -700e6 - 0.375, true);
  h.call(a, packet(12, b));
  b = new Uint8Array(40);
  b.set(context(h, a, s));
  v = new DataView(b.buffer);
  v.setUint32(24, 7, true);
  v.setBigUint64(32, 7n, true);
  h.call(a, packet(10, b));
  const bounds = h.call(a, packet(9, words(s)));
  assert.equal(u64(bounds, 72), document);
  assert.equal(u64(bounds, 80), 1n);
  assert.equal(u32(bounds, 88), 1);
  const view = new DataView(bounds.buffer);
  assert.deepEqual(
    [96, 104, 112, 120].map((offset) => view.getFloat64(offset, true)),
    [900e6 - 0.875, -700e6 - 1.375, 2, 2],
  );
  const op = queue(h, a, s, 2),
    w = take(h, a);
  h.copy(worker(h, w, 101).reply);
  const done = worker(h, w, 104, new Uint8Array([1, 0, 0, 0, 3, 0, 0, 0]));
  h.copy(done.reply);
  complete(h, a, done.result);
  h.call(a, packet(32, words(s, op)));
  const chooser = file(h, a, s, 1);
  assert.equal(u32(chooser, 232), 1);
  assert.equal(u32(chooser, 236), 2);
  b = new Uint8Array(40);
  b.set(context(h, a, s));
  v = new DataView(b.buffer);
  v.setBigUint64(24, u64(chooser, 240), true);
  v.setUint32(32, 1, true);
  assert.equal(u32(h.call(a, packet(21, b)), 232), 8);
  close(h, a, s);
}
