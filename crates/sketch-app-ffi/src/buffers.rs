use crate::{boundary::*, bridge};
/// # Safety
/// Pointer slots obey the v2 header preconditions.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ns_v2_buffer_len(b: u64, out: *mut usize, e: *mut NsV2Error) -> u32 {
    unsafe {
        run(e, || {
            let out = output(out)?;
            *out = bridge::buffer_len(b)?;
            Ok(())
        })
    }
}
/// # Safety
/// Destination is writable for capacity bytes and disjoint from scalar/error outputs.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ns_v2_buffer_copy(
    b: u64,
    offset: usize,
    dst: *mut u8,
    capacity: usize,
    out: *mut usize,
    e: *mut NsV2Error,
) -> u32 {
    unsafe {
        run(e, || {
            let out = output(out)?;
            if capacity > isize::MAX as usize || (dst.is_null() && capacity != 0) {
                return Err(invalid());
            }
            let destination = if capacity == 0 {
                &mut []
            } else {
                std::slice::from_raw_parts_mut(dst, capacity)
            };
            *out = bridge::buffer_copy(b, offset, destination)?;
            Ok(())
        })
    }
}
/// # Safety
/// Pointer slots obey the v2 header preconditions.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ns_v2_buffer_free(b: *mut u64, e: *mut NsV2Error) -> u32 {
    unsafe {
        run(e, || {
            let b = input(b)?;
            bridge::buffer_free(*b)?;
            *b = 0;
            Ok(())
        })
    }
}
