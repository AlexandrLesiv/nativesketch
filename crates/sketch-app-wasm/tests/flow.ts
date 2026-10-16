import assert from 'node:assert/strict';
import { packet, words, u64, u32, fixture } from './harness.ts';
import type { Harness } from './harness.ts';
export function session(h: Harness, a: number): bigint {
  return u64(h.call(a, fixture('create.cap3')), 72);
}
function context(h: Harness, a: number, s: bigint): Uint8Array {
  const r = h.call(a, packet(3, words(s)));
  return words(s, u64(r, 80), u64(r, 104));
}
function file(h: Harness, a: number, s: bigint, tag: number): Uint8Array {
  const b = new Uint8Array(32);
  b.set(context(h, a, s));
  new DataView(b.buffer).setUint32(24, tag, true);
  return h.call(a, packet(20, b));
}
export function queue(h: Harness, a: number, s: bigint, tag: number): bigint {
  const r = file(h, a, s, tag),
    b = new Uint8Array(64),
    v = new DataView(b.buffer);
  b.set(context(h, a, s));
  v.setBigUint64(24, u64(r, 240), true);
  v.setUint32(32, 1, true);
  v.setUint32(36, 7, true);
  v.setBigUint64(40, 0xfedcba9876543211n, true);
  v.setBigUint64(48, 1n, true);
  v.setBigUint64(56, 1n, true);
  return u64(h.call(a, packet(22, b)), 248);
}
export function take(h: Harness, a: number): number {
  return h.using(packet(30), (p) => {
    const o = h.invoke('app_take_work', [a, p]);
    h.copy(o.reply);
    return o.work;
  });
}
export function worker(
  h: Harness,
  w: number,
  op: number,
  body: Uint8Array = new Uint8Array(),
  input = 0,
) {
  return h.using(packet(op, body), (p) => h.invoke('work_call', [w, p, input]));
}
export function complete(h: Harness, a: number, r: number): void {
  h.using(packet(31), (p) => {
    const o = h.invoke('app_complete', [a, p, r]);
    assert.equal(o.flags, 4);
    h.copy(o.reply);
  });
}
export function close(h: Harness, a: number, s: bigint): void {
  const r = file(h, a, s, 6);
  assert.equal(u32(r, 232), 6);
  const b = new Uint8Array(40),
    v = new DataView(b.buffer);
  b.set(context(h, a, s));
  v.setBigUint64(24, u64(r, 240), true);
  v.setUint32(32, 5, true);
  assert.equal(u32(h.call(a, packet(21, b)), 232), 7);
  h.call(a, packet(2, words(s)));
  for (;;) {
    const w = take(h, a);
    if (!w) break;
    const o = worker(h, w, 106);
    assert.equal(o.flags, 1);
    h.copy(o.reply);
    complete(h, a, o.result);
  }
  while (u32(h.call(a, packet(34)), 72)) {}
  assert.equal(h.invoke('app_free', [a]).flags, 8);
}
export function checkWork(h: Harness): void {
  const a = h.create(),
    s = session(h, a),
    op = queue(h, a, s, 2),
    w = take(h, a);
  assert.ok(w);
  const encoded = worker(h, w, 101);
  assert.equal(encoded.flags, 0);
  assert.equal(u32(h.copy(encoded.reply), 72), 0);
  h.using(packet(104, new Uint8Array([4, 0, 0, 0, 0, 0, 0, 0])), (p) =>
    h.invoke('work_call', [w, p, 0], 8),
  );
  const o = worker(h, w, 104, new Uint8Array([1, 0, 0, 0, 3, 0, 0, 0]));
  assert.equal(o.flags, 1);
  assert.ok(o.result);
  h.copy(o.reply);
  h.invoke('result_retire_stopped', [o.result], 9);
  const foreign = h.create();
  h.using(packet(31), (p) => h.invoke('app_complete', [foreign, p, o.result], 11));
  h.invoke('app_free', [foreign]);
  complete(h, a, o.result);
  h.using(packet(31), (p) => h.invoke('app_complete', [a, p, o.result], 7));
  h.call(a, packet(32, words(s, op)));
  close(h, a, s);
}
export function checkRead(h: Harness): void {
  const a = h.create(),
    s = session(h, a),
    op = queue(h, a, s, 5),
    w = take(h, a);
  const input = h.using(
    new Uint8Array([1, 2, 3]),
    (p) => h.invoke('work_copy_input', [w, p]).primary,
  );
  h.using(packet(100), (p) => h.invoke('work_call', [w, p, input], 1));
  const o = worker(h, w, 102, new Uint8Array(), input);
  assert.equal(o.flags, 3);
  h.copy(o.reply);
  h.invoke('input_free', [input], 7);
  complete(h, a, o.result);
  const outcome = h.call(a, packet(32, words(s, op)));
  assert.equal(u32(outcome, 24), 0);
  assert.equal(u32(outcome, 232), 9);
  close(h, a, s);
}
