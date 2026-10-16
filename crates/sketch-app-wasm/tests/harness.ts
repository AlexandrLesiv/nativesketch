import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
export function fixture(name: string): Uint8Array {
  const data: unknown = JSON.parse(
    readFileSync(new URL('../../sketch-app-wire/tests/fixtures/v2.json', import.meta.url), 'utf8'),
  );
  assert.ok(data && typeof data === 'object');
  const hex: unknown = Reflect.get(data, name);
  assert.equal(typeof hex, 'string');
  if (typeof hex !== 'string') throw new Error('fixture');
  return new Uint8Array(Buffer.from(hex, 'hex'));
}
export function words(...values: bigint[]): Uint8Array {
  const b = new Uint8Array(values.length * 8);
  const v = new DataView(b.buffer);
  values.forEach((n, i) => v.setBigUint64(i * 8, n, true));
  return b;
}
export function packet(op: number, body: Uint8Array = new Uint8Array()): Uint8Array {
  const b = new Uint8Array(24 + body.length),
    v = new DataView(b.buffer);
  b.set([78, 83, 65, 80, 2]);
  v.setUint32(8, op, true);
  v.setUint32(12, body.length, true);
  v.setBigUint64(16, 0xfedcba9876543210n, true);
  b.set(body, 24);
  return b;
}
export function u64(b: Uint8Array, n: number): bigint {
  return new DataView(b.buffer, b.byteOffset, b.byteLength).getBigUint64(n, true);
}
export function u32(b: Uint8Array, n: number): number {
  return new DataView(b.buffer, b.byteOffset, b.byteLength).getUint32(n, true);
}
export function makeHarness(exports: WebAssembly.Exports) {
  const memory = requireMemory(exports.memory);
  function raw(name: string, ...args: number[]): number {
    const fn = exports['ns_v2_wasm_' + name];
    assert.equal(typeof fn, 'function');
    if (typeof fn !== 'function') throw new Error(name);
    const value: unknown = fn(...args);
    assert.equal(typeof value, 'number');
    if (typeof value !== 'number') throw new Error(name);
    return value;
  }
  function invoke(name: string, args: number[] = [], expected = 0) {
    assert.equal(raw(name, ...args), expected, name);
    const b = new Uint8Array(memory.buffer, raw('result_ptr'), 128).slice();
    assert.equal(u32(b, 0), expected);
    assert.deepEqual(b.slice(104), new Uint8Array(24));
    if (expected) {
      assert.deepEqual(b.slice(4, 32), new Uint8Array(28));
      assert.equal(u32(b, 56), expected);
    } else assert.deepEqual(b.slice(32, 104), new Uint8Array(72));
    return {
      errorFrame: b.slice(32, 104),
      flags: u32(b, 4),
      primary: u32(b, 8),
      reply: u32(b, 12),
      work: u32(b, 16),
      result: u32(b, 20),
      count: u32(b, 24),
    };
  }
  function upload(bytes: Uint8Array): number {
    const id = invoke('upload_new', [bytes.length]).primary;
    const view = invoke('upload_view', [id]);
    new Uint8Array(memory.buffer, view.primary, view.count).set(bytes);
    return id;
  }
  function using<T>(bytes: Uint8Array, fn: (id: number) => T): T {
    const id = upload(bytes);
    try {
      return fn(id);
    } finally {
      invoke('upload_free', [id]);
    }
  }
  function copy(reply: number): Uint8Array {
    const n = invoke('buffer_len', [reply]).count;
    const out = invoke('upload_new', [n]).primary;
    invoke('buffer_copy', [reply, 0, out, n]);
    const v = invoke('upload_view', [out]);
    const b = new Uint8Array(memory.buffer, v.primary, n).slice();
    invoke('upload_free', [out]);
    invoke('buffer_free', [reply]);
    return b;
  }
  function call(app: number, bytes: Uint8Array): Uint8Array {
    return using(bytes, (p) => copy(invoke('app_call', [app, p]).reply));
  }
  function create(): number {
    return using(fixture('bootstrap'), (p) => invoke('app_new', [p]).primary);
  }
  return { memory, raw, invoke, upload, using, copy, call, create };
}
export type Harness = ReturnType<typeof makeHarness>;

function requireMemory(value: WebAssembly.ExportValue): WebAssembly.Memory {
  if (!(value instanceof WebAssembly.Memory)) throw new Error('missing memory export');
  return value;
}
