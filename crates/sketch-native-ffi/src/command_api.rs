use crate::{boundary::*, convert, dto};
use sketch_core::application::{routing::*, *};
/// # Safety
/// Valid aligned buffers, unique handles and serialized owner-thread use are required.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sketch_session_v1_dispatch(
    pointer: *mut Runtime,
    handle: *mut Session,
    context: dto::Context,
    command: *const dto::Command,
    output: *mut dto::Outcome,
) -> u32 {
    unsafe {
        runtime(pointer, false, |value| {
            require_output(output)?;
            let session = session(handle, pointer)?;
            let context = status(value.core.context(
                session.window,
                context.session,
                context.request,
                context.interaction,
            ))?;
            let command = read(command)?;
            let result = if command.tag == 26 {
                if command.reserved != 0 {
                    return Err(1);
                }
                status(value.core.open_recovery(
                    session.window,
                    context,
                    command.payload.recovery_item,
                ))?
            } else {
                status(value.core.dispatch(
                    session.window,
                    context,
                    convert::commands::command(command)?,
                ))?
            };
            let projection = convert::projection::current(&value.core, session.window)?;
            output.write(convert::outcome::outcome(result, projection));
            Ok(())
        })
    }
}
/// # Safety
/// Valid aligned buffers, unique handles and serialized owner-thread use are required.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sketch_session_v1_input(
    pointer: *mut Runtime,
    handle: *mut Session,
    context: dto::Context,
    pointer_id: u64,
    samples: *const dto::Sample,
    count: u32,
    output: *mut dto::Outcome,
) -> u32 {
    unsafe {
        runtime(pointer, false, |value| {
            require_output(output)?;
            let session = session(handle, pointer)?;
            let context = status(value.core.context(
                session.window,
                context.session,
                context.request,
                context.interaction,
            ))?;
            let samples = convert::input::samples(slice(samples, count as usize, 4096)?)?;
            let result = status(value.core.dispatch(
                session.window,
                context,
                ApplicationCommand::Input(SampleBatch {
                    pointer: PointerId(pointer_id),
                    samples: &samples,
                }),
            ))?;
            let projection = convert::projection::current(&value.core, session.window)?;
            output.write(convert::outcome::outcome(result, projection));
            Ok(())
        })
    }
}
/// # Safety
/// Valid aligned buffers, unique handles and serialized owner-thread use are required.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sketch_session_v1_apply_event(
    pointer: *mut Runtime,
    handle: *mut Session,
    event: u64,
    output: *mut dto::Outcome,
) -> u32 {
    unsafe {
        runtime(pointer, false, |value| {
            require_output(output)?;
            let session = session(handle, pointer)?;
            let result = status(value.core.apply_event(session.window, event))?;
            let projection = convert::projection::current(&value.core, session.window)?;
            output.write(convert::outcome::outcome(result, projection));
            Ok(())
        })
    }
}
