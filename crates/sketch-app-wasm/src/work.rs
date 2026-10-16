use crate::*;
#[unsafe(no_mangle)]
pub extern "C" fn ns_v2_wasm_work_copy_input(work: u32, source: u32) -> u32 {
    run(|| {
        bridge::with_upload(source.into(), |s| {
            bridge::work_copy_input(work.into(), s).map(Output::primary)
        })
    })
}
#[unsafe(no_mangle)]
pub extern "C" fn ns_v2_wasm_work_call(work: u32, packet: u32, input: u32) -> u32 {
    run(|| {
        bridge::with_upload(packet.into(), |p| {
            let o = bridge::work_call(work.into(), p, input.into())?;
            Ok(Output {
                flags: u32::from(o.work_consumed) | (u32::from(o.input_consumed) << 1),
                reply: o.reply,
                result: o.result,
                ..Output::default()
            })
        })
    })
}
#[unsafe(no_mangle)]
pub extern "C" fn ns_v2_wasm_result_retire_stopped(result: u32) -> u32 {
    run(|| {
        bridge::result_retire_stopped(result.into())?;
        Ok(Output::consumed(4))
    })
}
#[unsafe(no_mangle)]
pub extern "C" fn ns_v2_wasm_input_free(input: u32) -> u32 {
    run(|| {
        bridge::input_free(input.into())?;
        Ok(Output::consumed(2))
    })
}
