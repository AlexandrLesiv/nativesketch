import assert from 'node:assert/strict';
import { packet, words, u64, u32 } from './harness.ts';
import type { Harness } from './harness.ts';
import { session, context, take, worker, complete, close } from './flow.ts';
function draw(h: Harness, a: number, s: bigint) {
  let b = new Uint8Array(48);
  b.set(context(h, a, s));
  let v = new DataView(b.buffer);
  v.setUint32(24, 6, true);
  v.setBigUint64(32, 0xfedcba9876543210n, true);
  v.setUint32(40, 2, true);
  h.call(a, packet(10, b));
  b = new Uint8Array(40 + 300 * 48);
  b.set(context(h, a, s));
  v = new DataView(b.buffer);
  v.setBigUint64(24, 0xfedcba9876543210n, true);
  v.setUint32(32, 300, true);
  for (let i = 0; i < 300; i++) {
    const n = 40 + i * 48;
    v.setBigUint64(n + 8, 0xf000000000000000n + BigInt(i), true);
    v.setFloat64(n + 16, i % 2 ? 999999010 : 0, true);
    v.setFloat64(n + 24, 20, true);
    v.setFloat64(n + 32, i, true);
  }
  h.call(a, packet(12, b));
  b = new Uint8Array(40);
  b.set(context(h, a, s));
  v = new DataView(b.buffer);
  v.setUint32(24, 7, true);
  v.setBigUint64(32, 0xfedcba9876543210n, true);
  h.call(a, packet(10, b));
}
function scene(h: Harness, a: number, s: bigint, x: number, width: number) {
  const b = new Uint8Array(56),
    v = new DataView(b.buffer);
  b.set(words(s, 0n));
  [x, 0, width, 100, 2].forEach((n, i) => v.setFloat64(16 + i * 8, n, true));
  return u64(h.call(a, packet(40, b)), 72);
}
export function checkViewport(h: Harness) {
  const a = h.create(),
    s = session(h, a),
    foreign = h.create();
  draw(h, a, s);
  const local = scene(h, a, s, 0, 1e9 + 100),
    b = new Uint8Array(24),
    v = new DataView(b.buffer);
  v.setBigUint64(0, local, true);
  v.setUint32(16, 133, true);
  v.setUint32(20, 137, true);
  let r = h.call(a, packet(45, b));
  assert.equal(r.length, 136);
  assert.equal(u32(r, 72), 0);
  assert.equal(u64(r, 80), 0n);
  assert.equal(u64(r, 120), 0n);
  h.call(a, packet(44, words(local)));
  const leased = scene(h, a, s, 999999000, 100);
  v.setBigUint64(0, leased, true);
  v.setUint32(16, 0, true);
  h.using(packet(45, b), (p) => h.invoke('app_call', [a, p], 1));
  v.setUint32(16, 133, true);
  v.setUint32(8, 1, true);
  v.setUint32(12, 1, true);
  h.using(packet(45, b), (p) => h.invoke('app_call', [a, p], 1));
  v.setUint32(8, 0, true);
  v.setUint32(12, 0, true);
  const held: number[] = [];
  for (let i = 0; i < 64; i++)
    held.push(h.using(packet(5), (p) => h.invoke('app_call', [a, p]).reply));
  h.using(packet(45, b), (p) => h.invoke('app_call', [a, p], 3));
  held.forEach((id) => h.invoke('buffer_free', [id]));
  r = h.call(a, packet(45, b));
  const prepared = u64(r, 80),
    count = Number(u64(r, 120));
  assert.equal(u32(r, 72), 1);
  assert.equal(u32(r, 76), 0);
  assert.equal(u64(r, 88), leased);
  assert.equal(u32(r, 128), 133);
  assert.equal(u32(r, 132), 137);
  assert.ok(count > 744);
  h.using(packet(45, b), (p) => h.invoke('app_call', [a, p], 9));
  h.call(a, packet(44, words(leased)));
  const page = new Uint8Array(16),
    pv = new DataView(page.buffer);
  page.set(words(prepared));
  pv.setUint32(12, 744, true);
  h.using(packet(46, page), (p) => h.invoke('app_call', [foreign, p], 7));
  const all = new Uint8Array(count * 88);
  let first = 0;
  while (first < count) {
    pv.setUint32(8, first, true);
    r = h.call(a, packet(46, page));
    const n = u32(r, 72);
    assert.ok(n > 0 && n <= 744);
    assert.equal(r.length, 80 + n * 88);
    assert.ok(r.length <= 65616);
    all.set(r.subarray(80), first * 88);
    first += n;
  }
  assert.equal(first, count);
  pv.setUint32(8, count, true);
  assert.equal(u32(h.call(a, packet(46, page)), 72), 0);
  pv.setUint32(8, count + 1, true);
  h.using(packet(46, page), (p) => h.invoke('app_call', [a, p], 1));
  // Independent output remains intact after source release and memory growth.
  h.memory.grow(1);
  pv.setUint32(8, 0, true);
  r = h.call(a, packet(46, page));
  assert.deepEqual(r.subarray(80), all.subarray(0, 744 * 88));
  const closing = new Uint8Array(32);
  closing.set(context(h, a, s));
  new DataView(closing.buffer).setUint32(24, 6, true);
  let closeReply = h.call(a, packet(20, closing));
  assert.equal(u32(closeReply, 232), 3);
  const decision = new Uint8Array(40),
    dv = new DataView(decision.buffer);
  decision.set(context(h, a, s));
  dv.setBigUint64(24, u64(closeReply, 240), true);
  dv.setUint32(32, 3, true);
  closeReply = h.call(a, packet(21, decision));
  assert.equal(u32(closeReply, 232), 6);
  decision.set(context(h, a, s));
  dv.setBigUint64(24, u64(closeReply, 240), true);
  dv.setUint32(32, 5, true);
  assert.equal(u32(h.call(a, packet(21, decision)), 232), 7);
  h.call(a, packet(2, words(s)));
  for (;;) {
    const w = take(h, a);
    if (!w) break;
    const done = worker(h, w, 106);
    h.copy(done.reply);
    complete(h, a, done.result);
  }
  h.invoke('app_free', [a], 9);
  h.using(packet(47, words(prepared)), (p) => h.invoke('app_call', [foreign, p], 7));
  h.call(a, packet(47, words(prepared)));
  h.using(packet(47, words(prepared)), (p) => h.invoke('app_call', [a, p], 7));
  h.using(packet(46, page), (p) => h.invoke('app_call', [a, p], 7));
  h.invoke('app_free', [a]);
  const fs = session(h, foreign);
  close(h, foreign, fs);
  console.log(
    'PASS Wasm viewport: local/physical modes, actual outlines, paging, growth, capacity, ownership, shutdown',
  );
}
