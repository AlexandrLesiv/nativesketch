use crate::registry::{LIMIT, registry};
#[unsafe(no_mangle)]
pub extern "C" fn sketch_wasm_buffer_new(length: u32) -> u32 {
    registry(|r| {
        if length as usize > LIMIT
            || r.buffers.values().map(Vec::len).sum::<usize>() + length as usize > 128 * 1024 * 1024
        {
            return 0;
        }
        r.buffer(vec![0; length as usize])
    })
}
#[unsafe(no_mangle)]
pub extern "C" fn sketch_wasm_buffer_ptr(token: u32) -> usize {
    registry(|r| r.buffers.get(&token).map_or(0, |b| b.as_ptr() as usize))
}
#[unsafe(no_mangle)]
pub extern "C" fn sketch_wasm_buffer_len(token: u32) -> u32 {
    registry(|r| r.buffers.get(&token).map_or(0, |b| b.len() as u32))
}
#[unsafe(no_mangle)]
pub extern "C" fn sketch_wasm_buffer_free(token: u32) -> u32 {
    registry(|r| {
        if r.editors.values().any(|s| s.output == token) {
            return 2;
        }
        if r.buffers.remove(&token).is_some() {
            0
        } else {
            1
        }
    })
}
