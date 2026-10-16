use crate::SketchEditor;
use crate::handles::edit;
macro_rules! unary {
    ($name:ident,$method:ident) => {
        /// # Safety
        /// Handle must be live with serialized access.
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn $name(e: *mut SketchEditor) -> u32 {
            unsafe { edit(e, |e| e.$method()) }
        }
    };
}
unary!(sketch_select_clear, select_clear);
unary!(sketch_delete_selected, delete_selected);
unary!(sketch_resize_begin, resize_begin);
unary!(sketch_resize_commit, resize_commit);
unary!(sketch_undo, undo);
unary!(sketch_redo, redo);
unary!(sketch_document_new, clear);
/// # Safety
/// Handle must be live with serialized access.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sketch_cancel(e: *mut SketchEditor) -> u32 {
    unsafe {
        edit(e, |e| {
            e.cancel();
            Ok(())
        })
    }
}
/// # Safety
/// Handle must be live with serialized access.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sketch_select_at(
    e: *mut SketchEditor,
    x: f64,
    y: f64,
    tolerance: f64,
) -> u32 {
    unsafe { edit(e, |e| e.select_at(x, y, tolerance)) }
}
/// # Safety
/// Handle must be live with serialized access.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sketch_select_cycle(e: *mut SketchEditor, direction: i32) -> u32 {
    unsafe { edit(e, |e| e.select_cycle(direction)) }
}
/// # Safety
/// Handle must be live with serialized access.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sketch_resize_update(
    e: *mut SketchEditor,
    scale: f64,
    x: f64,
    y: f64,
) -> u32 {
    unsafe { edit(e, |e| e.resize_update(scale, x, y)) }
}
