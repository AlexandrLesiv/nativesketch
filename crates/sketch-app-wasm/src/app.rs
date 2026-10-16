use crate::record::request;
use crate::*;
#[unsafe(no_mangle)]
pub extern "C" fn ns_v2_wasm_app_new(packet: u32) -> u32 {
    request(packet, |p| bridge::app_new(p).map(Output::primary))
}
#[unsafe(no_mangle)]
pub extern "C" fn ns_v2_wasm_app_call(app: u32, packet: u32) -> u32 {
    request(packet, |p| {
        bridge::app_call(app.into(), p).map(Output::reply)
    })
}
#[unsafe(no_mangle)]
pub extern "C" fn ns_v2_wasm_app_take_work(app: u32, packet: u32) -> u32 {
    request(packet, |p| {
        let (reply, work) = bridge::app_take_work(app.into(), p)?;
        Ok(Output {
            reply,
            work,
            ..Output::default()
        })
    })
}
#[unsafe(no_mangle)]
pub extern "C" fn ns_v2_wasm_app_complete(app: u32, packet: u32, result: u32) -> u32 {
    request(packet, |p| {
        let reply = bridge::app_complete(app.into(), p, result.into())?;
        Ok(Output {
            reply,
            flags: 4,
            ..Output::default()
        })
    })
}
#[unsafe(no_mangle)]
pub extern "C" fn ns_v2_wasm_app_free(app: u32) -> u32 {
    run(|| {
        bridge::app_free(app.into())?;
        Ok(Output::consumed(8))
    })
}
