use crate::registry::{Session, registry};
use sketch_core::Editor;
#[unsafe(no_mangle)]
pub extern "C" fn sketch_wasm_protocol_version() -> u32 {
    1
}
#[unsafe(no_mangle)]
pub extern "C" fn sketch_wasm_create() -> u32 {
    registry(|r| {
        let Some(id) = r.id() else { return 0 };
        r.editors.insert(
            id,
            Session {
                editor: Editor::new(),
                output: 0,
            },
        );
        id
    })
}
#[unsafe(no_mangle)]
pub extern "C" fn sketch_wasm_free(handle: u32) -> u32 {
    registry(|r| {
        let Some(s) = r.editors.remove(&handle) else {
            return 1;
        };
        r.buffers.remove(&s.output);
        0
    })
}
#[unsafe(no_mangle)]
pub extern "C" fn sketch_wasm_output(handle: u32) -> u32 {
    registry(|r| r.editors.get(&handle).map_or(0, |s| s.output))
}
