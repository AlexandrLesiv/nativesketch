use crate::{boundary::*, bridge};
/// # Safety
/// All pointers obey the v2 header preconditions. Input bytes are copied within bounds.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ns_v2_work_copy_input(
    w: u64,
    p: *const u8,
    n: usize,
    out: *mut u64,
    e: *mut NsV2Error,
) -> u32 {
    unsafe {
        run(e, || {
            let out = output(out)?;
            *out = bridge::work_copy_input(w, bytes(p, n)?)?;
            Ok(())
        })
    }
}
/// # Safety
/// All pointers obey the v2 header preconditions, including pairwise disjoint slots.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ns_v2_work_call(
    w: *mut u64,
    p: *const u8,
    n: usize,
    i: *mut u64,
    result: *mut u64,
    reply: *mut u64,
    e: *mut NsV2Error,
) -> u32 {
    unsafe {
        run_packet(e, p, n, |packet| {
            let result = output(result)?;
            let reply = output(reply)?;
            let w = input(w)?;
            let i = input(i)?;
            let outcome = bridge::work_call(*w, packet, *i)?;
            if outcome.work_consumed {
                *w = 0;
            }
            if outcome.input_consumed {
                *i = 0;
            }
            *result = outcome.result;
            *reply = outcome.reply;
            Ok(())
        })
    }
}
/// # Safety
/// Pointer slots obey the v2 header preconditions. This never acknowledges a save.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ns_v2_result_retire_stopped(r: *mut u64, e: *mut NsV2Error) -> u32 {
    unsafe {
        run(e, || {
            let r = input(r)?;
            bridge::result_retire_stopped(*r)?;
            *r = 0;
            Ok(())
        })
    }
}
/// # Safety
/// Pointer slots obey the v2 header preconditions.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ns_v2_input_free(i: *mut u64, e: *mut NsV2Error) -> u32 {
    unsafe {
        run(e, || {
            let i = input(i)?;
            bridge::input_free(*i)?;
            *i = 0;
            Ok(())
        })
    }
}
