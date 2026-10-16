//! No renderer rollout on other platforms is implied by the additive header.
use crate::{Runtime, Session, boundary};
use std::ffi::c_void;
/// # Safety
/// Output addresses writable handle storage. Metal is unsupported on this host.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sketch_renderer_v1_create(
    _: *const c_void,
    _: *const c_void,
    output: *mut *mut c_void,
) -> u32 {
    boundary::entry(|| {
        boundary::require_output(output)?;
        unsafe {
            output.write(std::ptr::null_mut());
        }
        Err(13)
    })
}
/// # Safety
/// Arguments obey the additive renderer header. No arguments are dereferenced here.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sketch_renderer_v1_draw(
    _: *mut Runtime,
    _: *mut Session,
    _: *mut c_void,
    _: *const c_void,
    _: *const c_void,
    _: *mut c_void,
) -> u32 {
    13
}
/// # Safety
/// Argument obeys the additive renderer header. No argument is dereferenced here.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sketch_renderer_v1_reset(_: *mut c_void) -> u32 {
    13
}
/// # Safety
/// Pointer addresses writable null handle storage; no renderer exists on this host.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sketch_renderer_v1_free(pointer: *mut *mut c_void) -> u32 {
    boundary::entry(|| {
        if unsafe { boundary::read(pointer)? }.is_null() {
            Ok(())
        } else {
            Err(13)
        }
    })
}
