import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { makeHarness, packet, fixture, u64, u32 } from './harness.ts';
import { checkPolicyBounds } from './policy.ts';
import { checkViewport } from './viewport.ts';
import { checkScale } from './scale.ts';
import { checkWork, checkRead, session, close } from './flow.ts';
const binary = process.argv[2];
assert.ok(binary, 'pass the actual Wasm path');
const { instance } = await WebAssembly.instantiate(readFileSync(binary), {});
const h = makeHarness(instance.exports);
const upload = h.upload(fixture('bootstrap')),
  view = h.invoke('upload_view', [upload]);
const stale = new Uint8Array(h.memory.buffer, view.primary, view.count);
h.memory.grow(2);
assert.equal(stale.byteLength, 0, 'memory.grow invalidates the former JS view');
const fresh = h.invoke('upload_view', [upload]);
assert.equal(fresh.count, view.count);
new Uint8Array(h.memory.buffer, fresh.primary, fresh.count).set(fixture('bootstrap'));
const a = h.invoke('app_new', [upload]).primary;
h.invoke('upload_free', [upload]);
h.invoke('upload_free', [upload], 7);
h.using(new Uint8Array([1, 2, 3]), (p) => h.invoke('app_call', [a, p], 1));
const failure = h.using(packet(0x70000005), (p) => h.invoke('app_call', [a, p], 1));
assert.equal(u32(failure.errorFrame, 8), 0xf0000005);
assert.equal(u64(failure.errorFrame, 16), 0xfedcba9876543210n);
const staleOwner = h.using(packet(5), (p) => h.invoke('app_call', [0, p], 7));
assert.equal(u64(staleOwner.errorFrame, 16), 0xfedcba9876543210n);
const s = session(h, a),
  replies: number[] = [];
for (let i = 0; i < 64; i++)
  replies.push(h.using(packet(5), (p) => h.invoke('app_call', [a, p]).reply));
h.using(fixture('create.cap3'), (p) => h.invoke('app_call', [a, p], 3));
const first = replies[0];
h.invoke('input_free', [first], 1);
h.invoke('app_free', [a], 9);
for (const r of replies) h.invoke('buffer_free', [r]);
assert.equal(u64(h.call(a, packet(5)), 72), 1n, 'failed create did not add a session');
close(h, a, s);
const ids: number[] = [];
for (let i = 0; i < 64; i++) ids.push(h.invoke('upload_new', [0]).primary);
h.invoke('upload_new', [0], 3);
for (const id of ids) h.invoke('upload_free', [id]);
h.invoke('upload_new', [64 * 1024 * 1024 + 2], 3);
checkWork(h);
checkRead(h);
checkPolicyBounds(h);
checkViewport(h);
checkScale(h);
console.log(
  'PASS Wasm v2: actual module, memory growth, bounded uploads/replies, real work/result, rejection and shutdown',
);
