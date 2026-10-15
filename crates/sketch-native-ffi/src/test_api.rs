use crate::{boundary::*, dto};
#[repr(C)]
#[derive(Clone, Copy)]
pub struct TestControl {
    pub disposable_control_root: dto::Text,
    pub stage: u32,
    pub mode: u32,
}
/// # Safety
/// Valid aligned buffers, unique handles and serialized owner-thread use are required.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sketch_runtime_v1_test_arm(
    pointer: *mut Runtime,
    config: *const TestControl,
) -> u32 {
    unsafe {
        runtime(pointer, false, |value| {
            let config = read(config)?;
            let control = sketch_services::native_runtime::TestControl::new(
                text(config.disposable_control_root, 16384)?.into(),
                config.stage,
                config.mode,
            )
            .map_err(|_| 1u32)?;
            status(value.core.arm_test(control))
        })
    }
}
