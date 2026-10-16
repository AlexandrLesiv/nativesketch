/** Legacy v1 conformance contract; production browsers use portable v2. */
export interface Api extends WebAssembly.Exports {
  memory: WebAssembly.Memory;
  sketch_wasm_protocol_version(): number;
  sketch_wasm_create(): number;
  sketch_wasm_free(h: number): number;
  sketch_wasm_buffer_new(n: number): number;
  sketch_wasm_buffer_ptr(t: number): number;
  sketch_wasm_buffer_len(t: number): number;
  sketch_wasm_buffer_free(t: number): number;
  sketch_wasm_command(h: number, op: number, input: number): number;
  sketch_wasm_output(h: number): number;
}
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
