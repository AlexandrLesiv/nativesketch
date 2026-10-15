import type { Api } from '../../apps/web/src/engine.ts';
/** Verify the actual module exports before exposing the shared browser ABI contract. */
export function wasmApi(exports: WebAssembly.Exports): Api {
  if (!(exports.memory instanceof WebAssembly.Memory)) throw new Error('Missing Wasm memory');
  for (const name of [
    'sketch_wasm_protocol_version',
    'sketch_wasm_create',
    'sketch_wasm_free',
    'sketch_wasm_buffer_new',
    'sketch_wasm_buffer_ptr',
    'sketch_wasm_buffer_len',
    'sketch_wasm_buffer_free',
    'sketch_wasm_command',
    'sketch_wasm_output',
  ]) {
    if (typeof exports[name] !== 'function') throw new Error(`Missing Wasm export: ${name}`);
  }
  return exports as Api;
}
