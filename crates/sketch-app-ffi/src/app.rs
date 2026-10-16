use crate::{boundary::*, bridge};
/// # Safety
/// All pointers obey the validity, disjointness and zero-output rules in the v2 header.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ns_v2_app_new(
    p: *const u8,
    n: usize,
    out: *mut u64,
    e: *mut NsV2Error,
) -> u32 {
    unsafe {
        run_packet(e, p, n, |packet| {
            let out = output(out)?;
            *out = bridge::app_new(packet)?;
            Ok(())
        })
    }
}
/// # Safety
/// All pointers obey the v2 header preconditions.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ns_v2_app_call(
    a: u64,
    p: *const u8,
    n: usize,
    out: *mut u64,
    e: *mut NsV2Error,
) -> u32 {
    unsafe {
        run_packet(e, p, n, |packet| {
            let out = output(out)?;
            *out = bridge::app_call(a, packet)?;
            Ok(())
        })
    }
}
/// # Safety
/// All pointers obey the v2 header preconditions; output slots are disjoint.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ns_v2_app_take_work(
    a: u64,
    p: *const u8,
    n: usize,
    w: *mut u64,
    r: *mut u64,
    e: *mut NsV2Error,
) -> u32 {
    unsafe {
        run_packet(e, p, n, |packet| {
            let w = output(w)?;
            let r = output(r)?;
            let (reply, work) = bridge::app_take_work(a, packet)?;
            *w = work;
            *r = reply;
            Ok(())
        })
    }
}
/// # Safety
/// All pointers obey the v2 header preconditions; result remains owned on rejection.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ns_v2_app_complete(
    a: u64,
    p: *const u8,
    n: usize,
    v: *mut u64,
    out: *mut u64,
    e: *mut NsV2Error,
) -> u32 {
    unsafe {
        run_packet(e, p, n, |packet| {
            let out = output(out)?;
            let v = input(v)?;
            let reply = bridge::app_complete(a, packet, *v)?;
            *v = 0;
            *out = reply;
            Ok(())
        })
    }
}
/// # Safety
/// Pointer slots obey the v2 header preconditions.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ns_v2_app_free(a: *mut u64, e: *mut NsV2Error) -> u32 {
    unsafe {
        run(e, || {
            let a = input(a)?;
            bridge::app_free(*a)?;
            *a = 0;
            Ok(())
        })
    }
}
