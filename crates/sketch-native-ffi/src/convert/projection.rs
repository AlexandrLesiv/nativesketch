use crate::dto;
use sketch_core::application::{routing::*, *};
use sketch_services::native_runtime::SessionSlot;
pub fn token(window: u64, value: EffectToken) -> dto::EffectToken {
    dto::EffectToken {
        owner: window,
        session: value.session().get(),
        serial: value.serial(),
        request: value.request().0,
    }
}
pub fn current(
    runtime: &sketch_services::native_runtime::NativeRuntime,
    window: u64,
) -> Result<dto::Projection, u32> {
    let slot = crate::boundary::status(runtime.session(window))?;
    let mut value = projection(slot);
    value.preparing_operations = crate::boundary::status(runtime.preparing_operations(window))?;
    Ok(value)
}
fn projection(slot: &SessionSlot) -> dto::Projection {
    let value = slot.app.projection();
    let state = value.editor;
    let brush = state.brush.brush();
    let (interaction_kind, interaction) = match state.interaction {
        InteractionState::Idle => (0, 0),
        InteractionState::Drawing(id) => (1, id.get()),
        InteractionState::Transforming(id) => (2, id.get()),
    };
    dto::Projection {
        window: slot.window,
        session: state.session.get(),
        revision: state.revision.0,
        scene_generation: state.scene_generation.get(),
        interaction,
        stroke_count: state.document.count as u64,
        selected_id: state.document.selected,
        destination: value.save.destination().map_or(0, |id| id.0),
        binding_generation: value.save.binding_generation(),
        binding_grant: slot.bound_grant,
        continuation: value
            .continuation
            .map_or_else(Default::default, |value| token(slot.window, value)),
        brush: dto::Brush {
            red: brush.rgba[0],
            green: brush.rgba[1],
            blue: brush.rgba[2],
            alpha: brush.rgba[3],
            width: brush.width,
        },
        tool: match state.tool {
            ToolId::Pen => 1,
            ToolId::Select => 2,
        },
        interaction_kind,
        capabilities: u32::from(state.capabilities.edit_document)
            | (u32::from(state.capabilities.export_document) << 1),
        dirty: state.document.dirty.into(),
        can_undo: state.document.can_undo.into(),
        can_redo: state.document.can_redo.into(),
        file_phase: match value.file {
            FilePhase::Idle => 0,
            FilePhase::Destination => 1,
            FilePhase::Source => 2,
            FilePhase::Unsaved => 3,
            FilePhase::Writes => 4,
            FilePhase::Replacement => 5,
            FilePhase::CloseReady => 6,
        },
        navigation: match value.navigation {
            None => 0,
            Some(NavigationIntent::New) => 1,
            Some(NavigationIntent::Open) => 2,
            Some(NavigationIntent::Recover(_)) => 3,
            Some(NavigationIntent::Close) => 4,
        },
        outstanding_saves: value.outstanding_saves,
        preparing_operations: 0,
        recovery_state: slot.recovery.as_ref().map_or(0, |value| value.state),
        reserved: 0,
    }
}
