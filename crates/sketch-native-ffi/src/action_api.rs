use crate::{boundary::*, convert, dto};
use sketch_core::application::{Availability, DisabledReason, routing::ApplicationCommand};
/// Validated high-level action through the same application owner as primitives.
/// # Safety
/// Valid aligned buffers, unique handles and serialized owner-thread use are required.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sketch_session_v1_action(
    pointer: *mut Runtime,
    handle: *mut Session,
    context: dto::Context,
    action: *const dto::EditorAction,
    output: *mut dto::Outcome,
) -> u32 {
    unsafe {
        runtime(pointer, false, |value| {
            require_output(output)?;
            let session = session(handle, pointer)?;
            let action = convert::actions::action(read(action)?)?;
            let context = status(value.core.context(
                session.window,
                context.session,
                context.request,
                context.interaction,
            ))?;
            let result = status(value.core.dispatch(
                session.window,
                context,
                ApplicationCommand::Action(action),
            ))?;
            let projection = convert::projection::current(&value.core, session.window)?;
            output.write(convert::outcome::outcome(result, projection));
            Ok(())
        })
    }
}
/// Advisory committed-state availability. Execution also validates context/geometry/capacity.
/// # Safety
/// Valid aligned buffers, unique handles and serialized owner-thread use are required.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sketch_session_v1_action_availability(
    pointer: *mut Runtime,
    handle: *mut Session,
    action: *const dto::EditorAction,
    output: *mut dto::Availability,
) -> u32 {
    unsafe {
        runtime(pointer, false, |value| {
            require_output(output)?;
            let session = session(handle, pointer)?;
            let action = convert::actions::action(read(action)?)?;
            let availability = status(value.core.session(session.window))?
                .app
                .projection()
                .editor
                .action_availability(action);
            let reason = match availability {
                Availability::Enabled => 0,
                Availability::Disabled(DisabledReason::ReadOnly) => 1,
                Availability::Disabled(DisabledReason::NothingSelected) => 4,
                Availability::Disabled(DisabledReason::HistoryEmpty) => 5,
                Availability::Disabled(_) => 2,
            };
            output.write(dto::Availability {
                enabled: u32::from(reason == 0),
                reason,
            });
            Ok(())
        })
    }
}
