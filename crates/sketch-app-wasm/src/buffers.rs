use crate::*;
#[unsafe(no_mangle)]
pub extern "C" fn ns_v2_wasm_upload_new(length: u32) -> u32 {
    run(|| bridge::upload_new(length as usize).map(Output::primary))
}
#[unsafe(no_mangle)]
pub extern "C" fn ns_v2_wasm_upload_view(upload: u32) -> u32 {
    run(|| {
        Ok(Output {
            primary: bridge::upload_pointer(upload.into())? as usize as u64,
            count: bridge::upload_len(upload.into())?,
            ..Output::default()
        })
    })
}
#[unsafe(no_mangle)]
pub extern "C" fn ns_v2_wasm_upload_free(upload: u32) -> u32 {
    run(|| {
        bridge::upload_free(upload.into())?;
        Ok(Output::consumed(32))
    })
}
#[unsafe(no_mangle)]
pub extern "C" fn ns_v2_wasm_buffer_len(buffer: u32) -> u32 {
    run(|| {
        Ok(Output {
            count: bridge::buffer_len(buffer.into())?,
            ..Output::default()
        })
    })
}
#[unsafe(no_mangle)]
pub extern "C" fn ns_v2_wasm_buffer_copy(
    buffer: u32,
    offset: u32,
    upload: u32,
    capacity: u32,
) -> u32 {
    run(|| {
        Ok(Output {
            count: bridge::upload_copy(
                buffer.into(),
                offset as usize,
                upload.into(),
                capacity as usize,
            )?,
            ..Output::default()
        })
    })
}
#[unsafe(no_mangle)]
pub extern "C" fn ns_v2_wasm_buffer_free(buffer: u32) -> u32 {
    run(|| {
        bridge::buffer_free(buffer.into())?;
        Ok(Output::consumed(16))
    })
}
