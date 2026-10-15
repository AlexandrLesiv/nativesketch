use crate::handles::edit;
use crate::{SketchBounds, SketchEditor, SketchState};
/// # Safety
/// The handle must be live and serialized; output must address writable state storage.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sketch_editor_state(
    editor: *mut SketchEditor,
    out: *mut SketchState,
) -> u32 {
    if out.is_null() {
        return 1;
    }
    unsafe {
        edit(editor, |e| {
            let s = e.state();
            *out = SketchState {
                stroke_count: s.count as u64,
                selected_id: s.selected,
                can_undo: u32::from(s.can_undo),
                can_redo: u32::from(s.can_redo),
                dirty: u32::from(s.dirty),
                interaction: s.interaction,
                selection: SketchBounds {
                    x: s.bounds.x,
                    y: s.bounds.y,
                    width: s.bounds.width,
                    height: s.bounds.height,
                },
            };
            Ok(())
        })
    }
}
