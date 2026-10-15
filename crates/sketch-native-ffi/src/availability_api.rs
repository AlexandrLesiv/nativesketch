use crate::{boundary::*, convert, dto};
use sketch_core::application::{Availability, DisabledReason};
/// # Safety
/// Valid aligned buffers, unique handles and serialized owner-thread use are required.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sketch_session_v1_availability(
    pointer: *mut Runtime,
    handle: *mut Session,
    action: u32,
    output: *mut dto::Availability,
) -> u32 {
    unsafe {
        runtime(pointer, false, |value| {
            require_output(output)?;
            let session = session(handle, pointer)?;
            let reason = if let Some(intent) = convert::commands::file(action) {
                match value.core.file_available(session.window, intent) {
                    Ok(()) => 0,
                    Err(sketch_services::native_runtime::RuntimeError::Application(
                        sketch_core::application::routing::ApplicationFailure::CapabilityDenied,
                    )) => 1,
                    Err(_) => 8,
                }
            } else if action == 26 {
                if status(value.core.session(session.window))?
                    .app
                    .projection()
                    .editor
                    .capabilities
                    .edit_document
                {
                    0
                } else {
                    1
                }
            } else {
                let action = convert::commands::availability(action).ok_or(1u32)?;
                match status(value.core.session(session.window))?
                    .app
                    .projection()
                    .editor
                    .availability(action)
                {
                    Availability::Enabled => 0,
                    Availability::Disabled(reason) => match reason {
                        DisabledReason::ReadOnly => 1,
                        DisabledReason::InteractionActive => 2,
                        DisabledReason::WrongTool => 3,
                        DisabledReason::NothingSelected => 4,
                        DisabledReason::HistoryEmpty => 5,
                        DisabledReason::NoActiveStroke => 6,
                        DisabledReason::NoActiveTransform => 7,
                    },
                }
            };
            output.write(dto::Availability {
                enabled: u32::from(reason == 0),
                reason,
            });
            Ok(())
        })
    }
}
