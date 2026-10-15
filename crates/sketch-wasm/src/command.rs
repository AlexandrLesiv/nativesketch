use crate::dispatch::command;
use crate::registry::REGISTRY;
use sketch_core::Error;
use std::panic::{AssertUnwindSafe, catch_unwind};
fn status(e: Error) -> u32 {
    match e {
        Error::InvalidArgument => 1,
        Error::InvalidState => 2,
        Error::Io => 4,
        Error::Format => 5,
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn sketch_wasm_command(handle: u32, opcode: u32, input: u32) -> u32 {
    // WASM production panics trap (never unwind); host must discard the instance.
    catch_unwind(AssertUnwindSafe(|| {
        REGISTRY.with(|cell| {
            let mut r = cell.borrow_mut();
            let bytes = if input == 0 {
                Vec::new()
            } else {
                match r.buffers.get(&input) {
                    Some(b) => b.clone(),
                    None => return 1,
                }
            };
            let Some(s) = r.editors.get_mut(&handle) else {
                return 1;
            };
            let old = s.output;
            s.output = 0;
            let result = command(&mut s.editor, opcode, &bytes);
            r.buffers.remove(&old);
            match result {
                Err(e) => status(e),
                Ok(bytes) => {
                    if !bytes.is_empty() {
                        let token = r.buffer(bytes);
                        if token == 0 {
                            return 3;
                        }
                        r.editors.get_mut(&handle).expect("live session").output = token;
                    }
                    0
                }
            }
        })
    }))
    .unwrap_or(6)
}
