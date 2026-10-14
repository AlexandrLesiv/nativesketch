//! C ABI for serialized, caller-owned editor handles. See the canonical header.
use sketch_core::{Brush, Editor, Error, Input, Sample};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::Path;
use std::ptr;

pub struct SketchEditor {
    editor: Editor,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SketchSample {
    pub x: f64,
    pub y: f64,
    pub timestamp: f64,
    pub pressure: f64,
    pub sample_id: u64,
    pub has_pressure: u32,
    pub kind: u32,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SketchBrush {
    pub red: f64,
    pub green: f64,
    pub blue: f64,
    pub alpha: f64,
    pub width: f64,
}
#[repr(C)]
#[derive(Default, Clone, Copy)]
pub struct SketchBounds {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}
#[repr(C)]
#[derive(Default, Clone, Copy)]
pub struct SketchState {
    pub stroke_count: u64,
    pub selected_id: u64,
    pub can_undo: u32,
    pub can_redo: u32,
    pub dirty: u32,
    pub interaction: u32,
    pub selection: SketchBounds,
}
#[repr(C)]
#[derive(Default, Clone, Copy)]
pub struct SketchPathCommand {
    pub stroke_id: u64,
    pub verb: u32,
    pub red: f64,
    pub green: f64,
    pub blue: f64,
    pub alpha: f64,
    pub x1: f64,
    pub y1: f64,
    pub x2: f64,
    pub y2: f64,
    pub x3: f64,
    pub y3: f64,
}
fn status(error: Error) -> u32 {
    match error {
        Error::InvalidArgument => 1,
        Error::InvalidState => 2,
        Error::Io => 4,
        Error::Format => 5,
    }
}
fn protect(f: impl FnOnce() -> Result<(), u32>) -> u32 {
    match catch_unwind(AssertUnwindSafe(f)) {
        Ok(Ok(())) => 0,
        Ok(Err(e)) => e,
        Err(_) => 6,
    }
}
// Safety: handle must be a live uniquely-owned handle; caller serializes all access.
unsafe fn edit(
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
/// # Safety
/// Handle must be live with serialized access.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sketch_stroke_begin(
    e: *mut SketchEditor,
    pointer: u64,
    device: u32,
    b: SketchBrush,
) -> u32 {
    unsafe {
        edit(e, |e| {
            e.begin(
                pointer,
                device,
                Brush {
                    rgba: [b.red, b.green, b.blue, b.alpha],
                    width: b.width,
                },
            )
        })
    }
}
/// # Safety
/// Handle must be live/serialized. `samples` must address `count` initialized samples,
/// borrowed for this call, or may be null when count is zero.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sketch_stroke_append(
    e: *mut SketchEditor,
    pointer: u64,
    samples: *const SketchSample,
    count: usize,
) -> u32 {
    unsafe {
        edit(e, |e| {
            if count > sketch_core::MAX_SAMPLES || count > 0 && samples.is_null() {
                return Err(Error::InvalidArgument);
            }
            let slice = if count == 0 {
                &[]
            } else {
                std::slice::from_raw_parts(samples, count)
            };
            let inputs: sketch_core::Result<Vec<_>> = slice
                .iter()
                .map(|s| {
                    if s.has_pressure > 1 || !s.pressure.is_finite() {
                        return Err(Error::InvalidArgument);
                    }
                    let p = Sample {
                        x: s.x,
                        y: s.y,
                        timestamp: s.timestamp,
                        pressure: if s.has_pressure == 1 {
                            Some(s.pressure)
                        } else {
                            None
                        },
                        id: s.sample_id,
                    };
                    match s.kind {
                        0 => Ok(Input::Actual(p)),
                        1 => Ok(Input::Predicted(p)),
                        2 => Ok(Input::Correction(p)),
                        _ => Err(Error::InvalidArgument),
                    }
                })
                .collect();
            e.append(pointer, &inputs?)
        })
    }
}
/// # Safety
/// Handle must be live with serialized access.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sketch_stroke_commit(e: *mut SketchEditor, pointer: u64) -> u32 {
    unsafe { edit(e, |e| e.commit(pointer)) }
}
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
// Safety: bytes must address len readable bytes. No borrow survives this call.
unsafe fn path<'a>(bytes: *const u8, len: usize) -> sketch_core::Result<&'a Path> {
    if bytes.is_null() || len == 0 || len > 32768 {
        return Err(Error::InvalidArgument);
    }
    let bytes = unsafe { std::slice::from_raw_parts(bytes, len) };
    if bytes.contains(&0) {
        return Err(Error::InvalidArgument);
    }
    Ok(Path::new(
        std::str::from_utf8(bytes).map_err(|_| Error::InvalidArgument)?,
    ))
}
/// # Safety
/// Handle must be live/serialized; path addresses length readable UTF-8 bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sketch_document_save(
    e: *mut SketchEditor,
    p: *const u8,
    len: usize,
) -> u32 {
    unsafe { edit(e, |e| e.save(path(p, len)?)) }
}
/// # Safety
/// Handle must be live/serialized; path addresses length readable UTF-8 bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sketch_document_load(
    e: *mut SketchEditor,
    p: *const u8,
    len: usize,
) -> u32 {
    unsafe { edit(e, |e| e.load(path(p, len)?)) }
}

/// # Safety
/// Live serialized handle; outputs writable; bytes addresses capacity writable bytes
/// or is null for a zero-capacity query. Input/output storage must not overlap.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sketch_document_export(
    e: *mut SketchEditor,
    bytes: *mut u8,
    capacity: usize,
    out_count: *mut usize,
    out_revision: *mut u64,
) -> u32 {
    protect(|| {
        if out_count.is_null() || out_revision.is_null() || (capacity > 0 && bytes.is_null()) {
            return Err(1);
        }
        let e = unsafe { e.as_ref() }.ok_or(1u32)?;
        let (snapshot, revision) = e.editor.export().map_err(status)?;
        unsafe {
            *out_count = snapshot.len();
            *out_revision = revision;
        }
        if bytes.is_null() && capacity == 0 {
            return Ok(());
        }
        if capacity < snapshot.len() {
            return Err(3);
        }
        unsafe {
            ptr::copy_nonoverlapping(snapshot.as_ptr(), bytes, snapshot.len());
        }
        Ok(())
    })
}
/// # Safety
/// Live serialized handle; bytes addresses length readable bytes borrowed for this call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sketch_document_import(
    e: *mut SketchEditor,
    bytes: *const u8,
    length: usize,
) -> u32 {
    if bytes.is_null() || length > 64 * 1024 * 1024 {
        return 1;
    }
    unsafe { edit(e, |e| e.import(std::slice::from_raw_parts(bytes, length))) }
}
/// # Safety
/// Live serialized handle; revision belongs to this handle's successful snapshot write.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sketch_document_mark_saved(e: *mut SketchEditor, revision: u64) -> u32 {
    unsafe { edit(e, |e| e.mark_saved(revision)) }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn byte_buffers_have_owned_copy_and_atomic_capacity_errors() {
        unsafe {
            let mut e = ptr::null_mut();
            assert_eq!(sketch_editor_create(&mut e), 0);
            let mut n = 0;
            let mut rev = 0;
            assert_eq!(
                sketch_document_export(e, ptr::null_mut(), 0, &mut n, &mut rev),
                0
            );
            let mut small = [255u8; 1];
            assert_eq!(
                sketch_document_export(e, small.as_mut_ptr(), 1, &mut n, &mut rev),
                3
            );
            assert_eq!(small, [255]);
            assert_eq!(
                sketch_document_export(e, ptr::null_mut(), 1, &mut n, &mut rev),
                1
            );
            let mut bytes = vec![0; n];
            assert_eq!(
                sketch_document_export(e, bytes.as_mut_ptr(), bytes.len(), &mut n, &mut rev),
                0
            );
            assert_eq!(sketch_document_import(e, ptr::null(), 0), 1);
            assert_eq!(sketch_document_import(e, b"bad".as_ptr(), 3), 5);
            assert_eq!(sketch_document_mark_saved(e, rev), 0);
            assert_eq!(sketch_document_import(e, bytes.as_ptr(), bytes.len()), 0);
            bytes.fill(0);
            assert_eq!(sketch_document_mark_saved(e, rev), 2);
            assert_eq!(
                sketch_document_export(e, ptr::null_mut(), 0, &mut n, &mut rev),
                0
            );
            assert_eq!(n, 24);
            sketch_editor_free(e);
        }
    }

    #[test]
    fn panic_is_contained() {
        assert_eq!(protect(|| panic!("injected panic")), 6);
    }
    #[test]
    fn ffi_errors_copy_ownership_and_real_editor_flow() {
        unsafe {
            assert_eq!(sketch_editor_create(ptr::null_mut()), 1);
            assert_eq!(sketch_cancel(ptr::null_mut()), 1);
            let mut e = ptr::null_mut();
            assert_eq!(sketch_editor_create(&mut e), 0);
            let b = SketchBrush {
                red: 0.0,
                green: 0.0,
                blue: 0.0,
                alpha: 1.0,
                width: 4.0,
            };
            assert_eq!(sketch_stroke_begin(e, 42, 0, b), 0);
            assert_eq!(sketch_stroke_append(e, 42, ptr::null(), 1), 1);
            let mut sample = SketchSample {
                x: 12.0,
                y: 20.0,
                timestamp: 1.0,
                pressure: 0.0,
                sample_id: 1,
                has_pressure: 2,
                kind: 0,
            };
            assert_eq!(sketch_stroke_append(e, 42, &sample, 1), 1);
            sample.has_pressure = 0;
            assert_eq!(sketch_stroke_append(e, 42, &sample, 1), 0);
            sample.x = 500.0;
            assert_eq!(sample.x, 500.0);
            assert_eq!(sketch_stroke_commit(e, 42), 0);
            let mut count = 0;
            assert_eq!(sketch_render_copy(e, ptr::null_mut(), 0, &mut count), 0);
            assert!(count > 0);
            let mut small = SketchPathCommand {
                stroke_id: 999,
                ..Default::default()
            };
            assert_eq!(sketch_render_copy(e, &mut small, 1, &mut count), 3);
            assert_eq!(small.stroke_id, 999);
            let mut output = vec![SketchPathCommand::default(); count];
            assert_eq!(
                sketch_render_copy(e, output.as_mut_ptr(), output.len(), &mut count),
                0
            );
            assert!(output.iter().all(|c| c.x1 < 100.0));
            let mut state = SketchState::default();
            assert_eq!(sketch_editor_state(e, &mut state), 0);
            assert_eq!(state.stroke_count, 1);
            assert_eq!(sketch_undo(e), 0);
            assert_eq!(sketch_redo(e), 0);
            assert_eq!(sketch_document_load(e, b"\xff".as_ptr(), 1), 1);
            assert_eq!(sketch_editor_state(e, ptr::null_mut()), 1);
            sketch_editor_free(e);
            sketch_editor_free(ptr::null_mut());
        }
    }
}
