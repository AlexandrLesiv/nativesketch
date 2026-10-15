use crate::{boundary::*, convert, dto};
use sketch_core::application::routing::*;
/// # Safety
/// Valid aligned buffers, unique handles and serialized owner-thread use are required.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sketch_session_v1_reply(
    pointer: *mut Runtime,
    handle: *mut Session,
    context: dto::Context,
    token: dto::EffectToken,
    reply: u32,
    output: *mut dto::Outcome,
) -> u32 {
    unsafe {
        runtime(pointer, false, |value| {
            require_output(output)?;
            let session = session(handle, pointer)?;
            if token.owner != session.window {
                return Err(7);
            }
            let context = status(value.core.context(
                session.window,
                context.session,
                context.request,
                context.interaction,
            ))?;
            let token = status(value.core.effect_token(
                session.window,
                token.session,
                token.serial,
                token.request,
            ))?;
            let decision = match reply {
                1 => EffectDecision::Cancel,
                2 => EffectDecision::Unsaved(UnsavedChoice::Save),
                3 => EffectDecision::Unsaved(UnsavedChoice::Discard),
                4 => EffectDecision::Continue,
                5 => EffectDecision::ConfirmClose,
                _ => return Err(1),
            };
            let result = status(value.core.reply(session.window, context, token, decision))?;
            output.write(convert::outcome::outcome(
                result,
                convert::projection::current(&value.core, session.window)?,
            ));
            Ok(())
        })
    }
}
/// # Safety
/// Valid aligned buffers, unique handles and serialized owner-thread use are required.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sketch_session_v1_supply_location(
    pointer: *mut Runtime,
    handle: *mut Session,
    context: dto::Context,
    token: dto::EffectToken,
    location: *const dto::Location,
    output: *mut u64,
) -> u32 {
    unsafe {
        runtime(pointer, false, |value| {
            require_output(output)?;
            let session = session(handle, pointer)?;
            if token.owner != session.window {
                return Err(7);
            }
            let context = status(value.core.context(
                session.window,
                context.session,
                context.request,
                context.interaction,
            ))?;
            let token = status(value.core.effect_token(
                session.window,
                token.session,
                token.serial,
                token.request,
            ))?;
            let location = read(location)?;
            if location.provider != 0 {
                return Err(13);
            }
            if location.reserved != 0 {
                return Err(1);
            }
            let path = text(location.path, 16384)?.into();
            output.write(status(value.core.supply_location(
                session.window,
                context,
                token,
                location.grant_id,
                path,
            ))?);
            Ok(())
        })
    }
}
