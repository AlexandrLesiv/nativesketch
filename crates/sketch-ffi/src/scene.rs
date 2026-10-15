use crate::status::protect;
use crate::{SketchEditor, SketchPathCommand};
/// # Safety
/// Handle must be live/serialized. `out_count` addresses writable size storage.
/// `commands` addresses `capacity` writable commands; null is allowed only for capacity zero.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sketch_render_copy(
    e: *mut SketchEditor,
    commands: *mut SketchPathCommand,
    capacity: usize,
    out_count: *mut usize,
) -> u32 {
    protect(|| {
        if out_count.is_null() || capacity > 0 && commands.is_null() {
            return Err(1);
        }
        let e = unsafe { e.as_ref() }.ok_or(1u32)?;
        let rendered = e.editor.render();
        unsafe {
            *out_count = rendered.len();
        }
        if capacity == 0 && commands.is_null() {
            return Ok(());
        }
        if capacity < rendered.len() {
            return Err(3);
        }
        for (i, c) in rendered.iter().enumerate() {
            let p = c.points;
            unsafe {
                commands.add(i).write(SketchPathCommand {
                    stroke_id: c.stroke_id,
                    verb: c.verb,
                    red: c.rgba[0],
                    green: c.rgba[1],
                    blue: c.rgba[2],
                    alpha: c.rgba[3],
                    x1: p[0],
                    y1: p[1],
                    x2: p[2],
                    y2: p[3],
                    x3: p[4],
                    y3: p[5],
                });
            }
        }
        Ok(())
    })
}
