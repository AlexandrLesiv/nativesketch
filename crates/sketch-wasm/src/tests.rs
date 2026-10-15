use super::*;
#[test]
fn malformed_messages_and_handles_are_errors() {
    assert_eq!(sketch_wasm_command(u32::MAX, 1, 0), 1);
    let h = sketch_wasm_create();
    let b = sketch_wasm_buffer_new(3);
    assert_eq!(sketch_wasm_command(h, 2, b), 1);
    assert_eq!(sketch_wasm_command(h, 1, 999999), 1);
    assert_eq!(sketch_wasm_command(h, 1, 0), 0);
    let out = sketch_wasm_output(h);
    assert_eq!(sketch_wasm_buffer_len(out), 64);
    assert_eq!(sketch_wasm_buffer_free(out), 2);
    assert_eq!(sketch_wasm_free(h), 0);
    assert_eq!(sketch_wasm_buffer_len(out), 0);
    assert_eq!(sketch_wasm_free(h), 1);
    assert_eq!(sketch_wasm_buffer_free(b), 0);
}
