use crate::status::{protect, status};
use sketch_core::Editor;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::ptr;
pub struct SketchEditor {
    pub(crate) editor: Editor,
}
// Safety: handle must be a live uniquely-owned handle; caller serializes all access.
pub(crate) unsafe fn edit(
    handle: *mut SketchEditor,
    f: impl FnOnce(&mut Editor) -> sketch_core::Result<()>,
) -> u32 {
    protect(|| {
        let handle = unsafe { handle.as_mut() }.ok_or(1u32)?;
        f(&mut handle.editor).map_err(status)
    })
}
#[unsafe(no_mangle)]
pub extern "C" fn sketch_abi_version() -> u32 {
    2
}
/// # Safety
/// `out_editor` must address writable pointer storage. The returned handle must be freed once.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sketch_editor_create(out_editor: *mut *mut SketchEditor) -> u32 {
    protect(|| {
        if out_editor.is_null() {
            return Err(1);
        }
        unsafe {
            *out_editor = ptr::null_mut();
            *out_editor = Box::into_raw(Box::new(SketchEditor {
                editor: Editor::new(),
            }));
        }
        Ok(())
    })
}
/// # Safety
/// `editor` must be null or an unfreed handle. No concurrent calls are permitted.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sketch_editor_free(editor: *mut SketchEditor) {
    let _ = catch_unwind(AssertUnwindSafe(|| {
        if !editor.is_null() {
            drop(unsafe { Box::from_raw(editor) });
        }
    }));
}
