use crate::SketchEditor;
use crate::handles::edit;
use crate::status::{protect, status};
use sketch_core::Error;
use std::path::Path;
use std::ptr;
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
