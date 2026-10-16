import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { packet, words, u64, fixture } from './harness.ts';
import type { Harness } from './harness.ts';
import { context, session, close } from './flow.ts';

// ANDROID-SCALE-001, UNDO-001/002, FFI-001: actual Wasm and shared literal packet.
export function checkScale(h: Harness): void {
  const a = h.create(),
    s = session(h, a);
  function command(op: number, tag: number, tail: Uint8Array = new Uint8Array()): Uint8Array {
    const b = new Uint8Array(32 + tail.length);
    b.set(context(h, a, s));
    new DataView(b.buffer).setUint32(24, tag, true);
    b.set(tail, 32);
    return h.call(a, packet(op, b));
  }
  command(10, 6, words(7n, 2n));
  const sample = fixture('sample.0.0').slice(24);
  sample.set(context(h, a, s));
  const sampleView = new DataView(sample.buffer);
  sampleView.setFloat64(56, 20, true);
  sampleView.setFloat64(64, 30, true);
  h.call(a, packet(12, sample));
  command(10, 7, words(7n));
  command(11, 2, words(2n));
  function bounds(): number[] {
    const r = h.call(a, packet(6, words(s))),
      v = new DataView(r.buffer);
    return [72, 80, 88, 96].map((offset) => v.getFloat64(offset, true));
  }
  const initial = bounds();
  const request = new Uint8Array(
    readFileSync(
      new URL('../../sketch-app-wire/tests/fixtures/action-scale-top-left.bin', import.meta.url),
    ),
  );
  function scale(factor = 1.5): Uint8Array {
    request.set(context(h, a, s), 24);
    new DataView(request.buffer).setFloat64(64, factor, true);
    return h.call(a, request);
  }
  command(10, 8);
  const transform = new Uint8Array(24),
    v = new DataView(transform.buffer);
  [4, -100, -200].forEach((value, i) => v.setFloat64(i * 8, value, true));
  command(10, 9, transform);
  const preview = bounds();
  request.set(context(h, a, s), 24);
  new DataView(request.buffer).setFloat64(64, Number.NaN, true);
  const state = h.call(a, packet(3, words(s)));
  h.using(request, (p) => h.invoke('app_call', [a, p], 1));
  assert.deepEqual(h.call(a, packet(3, words(s))), state);
  assert.deepEqual(bounds(), preview);
  const changed = scale();
  const expected = (factor: number) => [
    initial[0],
    initial[1],
    initial[2] * factor,
    initial[3] * factor,
  ];
  assert.equal(u64(changed, 104), 0n);
  assert.deepEqual(bounds(), expected(1.5));
  assert.equal(u64(scale(1), 88), u64(changed, 88));
  scale(); // same request ID: relative actions are not deduplicated
  assert.deepEqual(bounds(), expected(2.25));
  command(11, 4);
  assert.deepEqual(bounds(), expected(1.5));
  command(11, 4);
  assert.deepEqual(bounds(), initial);
  assert.equal(u64(command(11, 4), 112), 0n);
  close(h, a, s);
  console.log(
    'PASS Wasm top-left scale: literal packet, preview validation, relative repeat and undo',
  );
}
