// Fixed protocol 1 baseline; actual Wasm execution, not a JavaScript model.
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';

const fixture = JSON.parse(
  readFileSync(new URL('./fixtures/protocol-v1.json', import.meta.url), 'utf8'),
);
const artifact = process.argv[2];
assert.ok(artifact, 'usage: node protocol-compatibility.mjs <sketch_wasm.wasm>');
const { instance } = await WebAssembly.instantiate(readFileSync(artifact), {});
const w = instance.exports;
assert.equal(w.sketch_wasm_protocol_version(), fixture.protocol);
const editor = w.sketch_wasm_create();
assert.notEqual(editor, 0);

try {
  for (const step of fixture.steps) {
    const input = Buffer.from(step.inputHex, 'hex');
    const token = input.length ? w.sketch_wasm_buffer_new(input.length) : 0;
    assert.ok(!input.length || token, step.name);
    if (token) {
      new Uint8Array(w.memory.buffer, w.sketch_wasm_buffer_ptr(token), input.length).set(input);
    }
    try {
      assert.equal(w.sketch_wasm_command(editor, step.opcode, token), step.status, step.name);
      const output = w.sketch_wasm_output(editor);
      // Copy before the next call can grow memory or invalidate the output token.
      const bytes = output
        ? Buffer.from(new Uint8Array(
          w.memory.buffer,
          w.sketch_wasm_buffer_ptr(output),
          w.sketch_wasm_buffer_len(output),
        ))
        : Buffer.alloc(0);
      assert.equal(bytes.toString('hex'), step.outputHex, step.name);
      if (step.name === 'fixed-snapshot') {
        const documentPath = new URL(
          '../../sketch-core/tests/fixtures/portable-v1.nsketch', import.meta.url,
        );
        assert.deepEqual(bytes.subarray(8), readFileSync(fileURLToPath(documentPath)));
      }
    } finally {
      if (token) assert.equal(w.sketch_wasm_buffer_free(token), 0);
    }
  }
} finally {
  assert.equal(w.sketch_wasm_free(editor), 0);
}
console.log(`PASS: protocol-v1-fixed-fixture (${fixture.steps.length} request/response pairs)`);
