import { wasmApi } from '../../../scripts/lib/wasm-api.ts';
import { readFileSync } from 'node:fs';
import assert from 'node:assert/strict';
const { instance } = await WebAssembly.instantiate(readFileSync(process.argv[2]), {});
const w = wasmApi(instance.exports);
assert.equal(w.sketch_wasm_protocol_version(), 1);
const h = w.sketch_wasm_create();
assert.ok(h);
const payload = (size: number, write: (view: DataView) => void) => {
  const bytes = new Uint8Array(size);
  write(new DataView(bytes.buffer));
  return bytes;
};
const u64 = (n: number | bigint) => payload(8, (v) => v.setBigUint64(0, BigInt(n), true));
function call(op: number, bytes = new Uint8Array(), expected = 0, editor = h) {
  const token = bytes.length ? w.sketch_wasm_buffer_new(bytes.length) : 0;
  if (token)
    new Uint8Array(w.memory.buffer, w.sketch_wasm_buffer_ptr(token), bytes.length).set(bytes);
  assert.equal(w.sketch_wasm_command(editor, op, token), expected, `opcode ${op}`);
  if (token) assert.equal(w.sketch_wasm_buffer_free(token), 0);
  const output = w.sketch_wasm_output(editor);
  return output
    ? new Uint8Array(
        w.memory.buffer,
        w.sketch_wasm_buffer_ptr(output),
        w.sketch_wasm_buffer_len(output),
      ).slice()
    : new Uint8Array();
}
const state = () => new DataView(call(1).buffer);
const begin = () =>
  call(
    2,
    payload(52, (v) => {
      v.setBigUint64(0, 42n, true);
      v.setUint32(8, 1, true);
      [0.2, 0.3, 0.4, 0.5, 6].forEach((x, i) => v.setFloat64(12 + i * 8, x, true));
    }),
  );
const sample = (values: number[], kind = 0) =>
  payload(56, (v) => {
    v.setBigUint64(0, 42n, true);
    values.forEach((x, i) => v.setFloat64(8 + i * 8, x, true));
    v.setBigUint64(40, BigInt(values[2]), true);
    v.setUint32(48, 1, true);
    v.setUint32(52, kind, true);
  });
assert.equal(state().getBigUint64(0, true), 0n);
begin();
call(3, sample([10, 20, 1, 0.5]));
call(3, sample([30, 40, 2, 0.7]));
call(3, sample([80, 90, 3, 0.9], 1));
call(17, new Uint8Array(), 2);
call(4, u64(42));
assert.equal(state().getBigUint64(0, true), 1n);
assert.equal(state().getUint32(24, true), 1);
const snapshot = call(17);
const revision = new DataView(snapshot.buffer).getBigUint64(0, true);
assert.deepEqual(
  Buffer.from(snapshot.slice(8)),
  readFileSync(process.argv[3]),
  'native ABI and WASM bytes are identical; predictions omitted',
);
const geometry = call(6);
assert.equal(geometry.length % 96, 0);
assert.ok(geometry.length > 0);
for (let i = 0; i < geometry.length; i += 96) {
  const v = new DataView(geometry.buffer, i, 96);
  assert.equal(v.getBigUint64(0, true), 1n);
  assert.equal(v.getUint32(12, true), 0);
}
call(18, new Uint8Array([1, 2, 3]), 5);
assert.equal(state().getBigUint64(0, true), 1n);
call(19, u64(revision));
assert.equal(state().getUint32(24, true), 0);
call(14);
assert.equal(state().getBigUint64(0, true), 0n);
call(15);
assert.equal(state().getUint32(24, true), 0);
call(
  8,
  payload(4, (v) => v.setInt32(0, 1, true)),
);
call(11);
call(
  12,
  payload(24, (v) => {
    v.setFloat64(0, 2, true);
    v.setFloat64(8, 0, true);
    v.setFloat64(16, 0, true);
  }),
);
call(5);
assert.deepEqual(call(6), geometry, 'cancel resize preserves geometry');
call(10);
assert.equal(state().getBigUint64(0, true), 0n);
call(14);
begin();
call(3, sample([20, 20, 1, 0.4]));
call(5);
assert.equal(state().getBigUint64(0, true), 1n);
call(16);
call(19, u64(revision), 2);
call(18, snapshot.slice(8));
call(19, u64(revision), 2);
assert.deepEqual(call(6), geometry, 'native bytes restore same geometry');
call(2, new Uint8Array(51), 1);
call(1, new Uint8Array([0]), 1);
assert.equal(w.sketch_wasm_command(h, 1, 0xffffffff), 1);
assert.equal(w.sketch_wasm_command(0xffffffff, 1, 0), 1);
assert.equal(w.sketch_wasm_buffer_new(0xffffffff), 0);
call(1);
const old = w.sketch_wasm_output(h);
assert.equal(w.sketch_wasm_buffer_free(old), 2);
call(1);
assert.equal(w.sketch_wasm_buffer_ptr(old), 0);
assert.equal(w.sketch_wasm_free(h), 0);
assert.equal(w.sketch_wasm_free(h), 1);
console.log(
  'PASS: real Node WebAssembly draw/prediction/cancel/resize/delete/undo/redo, native byte conformance, stale saves, invalid documents/handles/buffers, output lifetime',
);
