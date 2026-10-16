use crate::bridge;
use bridge::{Failure, Result, WireError};
#[repr(C)]
pub struct NsV2Error {
    pub bytes: [u8; 72],
}
pub(crate) fn invalid() -> WireError {
    WireError {
        status: 1,
        failure: Failure::default(),
    }
}
pub(crate) unsafe fn run(error: *mut NsV2Error, f: impl FnOnce() -> Result<()>) -> u32 {
    if error.is_null() {
        return 1;
    }
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)).unwrap_or_else(|_| {
        Err(WireError {
            status: 6,
            failure: Failure::default(),
        })
    });
    let (status, bytes) = match result {
        Ok(()) => (0, [0; 72]),
        Err(e) => (e.status, e.packet(0, 0)),
    };
    // Caller guarantees a writable, aligned error record, disjoint from other arguments.
    unsafe {
        (*error).bytes = bytes;
    }
    status
}
pub(crate) unsafe fn bytes<'a>(p: *const u8, len: usize) -> Result<&'a [u8]> {
    if len > isize::MAX as usize || (p.is_null() && len != 0) {
        return Err(invalid());
    }
    if len == 0 {
        return Ok(&[]);
    }
    Ok(unsafe { std::slice::from_raw_parts(p, len) })
}
pub(crate) unsafe fn output<'a, T: Default + PartialEq>(p: *mut T) -> Result<&'a mut T> {
    let value = unsafe { p.as_mut() }.ok_or_else(invalid)?;
    if *value != T::default() {
        return Err(invalid());
    }
    Ok(value)
}
pub(crate) unsafe fn input<'a>(p: *mut u64) -> Result<&'a mut u64> {
    unsafe { p.as_mut() }.ok_or_else(invalid)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn panic_is_a_fixed_error_without_cross_boundary_unwind() {
        let mut error = NsV2Error { bytes: [91; 72] };
        let status = unsafe { run(&mut error, || panic!("boundary test")) };
        assert_eq!(status, 6);
        assert_eq!(&error.bytes[..4], b"NSAP");
        assert_eq!(&error.bytes[24..28], &6u32.to_le_bytes());
    }
}
