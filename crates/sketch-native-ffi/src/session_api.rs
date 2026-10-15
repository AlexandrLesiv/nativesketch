use crate::{boundary::*, convert, dto};
use sketch_core::application::Capabilities;
/// # Safety
/// Valid aligned buffers, unique handles and serialized owner-thread use are required.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sketch_session_v1_create(
    pointer: *mut Runtime,
    config: *const dto::SessionConfig,
    output: *mut *mut Session,
) -> u32 {
    unsafe {
        runtime(pointer, false, |value| {
            require_output(output)?;
            output.write(std::ptr::null_mut());
            let config = read(config)?;
            if config.capabilities & !3 != 0 {
                return Err(1);
            }
            let window = status(value.core.create(
                Capabilities {
                    edit_document: config.capabilities & 1 != 0,
                    export_document: config.capabilities & 2 != 0,
                },
                convert::commands::tool(config.tool)?,
                convert::commands::brush(config.brush)?,
            ))?;
            output.write(Box::into_raw(Box::new(Session {
                runtime: pointer,
                window,
                scene_active: Default::default(),
            })));
            Ok(())
        })
    }
}
/// # Safety
/// Valid aligned buffers, unique handles and serialized owner-thread use are required.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sketch_session_v1_projection(
    pointer: *mut Runtime,
    handle: *mut Session,
    output: *mut dto::Projection,
) -> u32 {
    unsafe {
        runtime(pointer, false, |value| {
            require_output(output)?;
            let session = session(handle, pointer)?;
            output.write(convert::projection::projection(status(
                value.core.session(session.window),
            )?));
            Ok(())
        })
    }
}
/// # Safety
/// Valid aligned buffers, unique handles and serialized owner-thread use are required.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sketch_session_v1_detach(
    pointer: *mut Runtime,
    handle: *mut *mut Session,
) -> u32 {
    unsafe {
        runtime(pointer, true, |value| {
            let raw = read(handle)?;
            let session = session(raw, pointer)?;
            status(value.core.detach(session.window, value.poisoned))?;
            drop(Box::from_raw(raw));
            handle.write(std::ptr::null_mut());
            Ok(())
        })
    }
}
