mod dispatch;
mod files;
mod navigation;
mod replacement;
mod replies;
mod types;
use super::{storage::*, *};
pub use replacement::{
    PreparedReplacement, ReplacementFailure, ReplacementSource, ReplacementTicket,
};
use std::sync::atomic::AtomicU64;
pub use types::*;
static NEXT_EFFECT: AtomicU64 = AtomicU64::new(1);
#[derive(Debug, Clone, Copy)]
enum Pending {
    Destination(EffectToken, SavePurpose),
    Source(EffectToken, Revision),
    Unsaved(EffectToken, Revision),
    Writes(EffectToken),
    Replacement(EffectToken, Revision),
    CloseReady(EffectToken, Revision),
}
impl Pending {
    fn token(self) -> EffectToken {
        match self {
            Self::Destination(t, _)
            | Self::Source(t, _)
            | Self::Unsaved(t, _)
            | Self::Writes(t)
            | Self::Replacement(t, _)
            | Self::CloseReady(t, _) => t,
        }
    }
    fn phase(self) -> FilePhase {
        match self {
            Self::Destination(..) => FilePhase::Destination,
            Self::Source(..) => FilePhase::Source,
            Self::Unsaved(..) => FilePhase::Unsaved,
            Self::Writes(..) => FilePhase::Writes,
            Self::Replacement(..) => FilePhase::Replacement,
            Self::CloseReady(..) => FilePhase::CloseReady,
        }
    }
}
#[derive(Debug, Clone, Copy)]
struct Navigation {
    intent: NavigationIntent,
    save: Option<SaveToken>,
}
/// Portable semantic owner. Native presentation and filesystem effects remain outside this type.
#[derive(Debug)]
pub struct ApplicationSession {
    editor: EditorSession,
    pending: Option<Pending>,
    navigation: Option<Navigation>,
    saves: Vec<SaveToken>,
}
impl ApplicationSession {
    pub fn new(
        capabilities: Capabilities,
        tool: ToolId,
        brush: BrushSettings,
        initial: Option<ValidatedDocument>,
    ) -> Result<Self, EditorError> {
        Ok(Self {
            editor: EditorSession::new(capabilities, tool, brush, initial)?,
            pending: None,
            navigation: None,
            saves: Vec::with_capacity(1024),
        })
    }
    pub fn projection(&self) -> ApplicationProjection {
        ApplicationProjection {
            editor: self.editor.view_state(),
            save: self.editor.save_state(),
            file: self.pending.map_or(FilePhase::Idle, Pending::phase),
            continuation: self.pending.map(Pending::token),
            navigation: self.navigation.map(|n| n.intent),
            outstanding_saves: self.saves.len() as u32,
        }
    }
    pub fn query(&self, query: EditorQuery) -> Result<QueryResult, EditorError> {
        self.editor.query(query)
    }
    fn explicit_pending(&self) -> bool {
        self.saves
            .iter()
            .any(|token| token.purpose() != SavePurpose::Checkpoint)
    }
    fn issue(&self, request: RequestId) -> Result<EffectToken, ApplicationFailure> {
        let serial =
            super::ids::reserve(&NEXT_EFFECT).map_err(|_| ApplicationFailure::ResourceLimit)?;
        Ok(EffectToken {
            session: self.editor.view_state().session,
            serial,
            request,
        })
    }
    fn cancel_interaction(&mut self, context: CommandContext) -> Result<(), ApplicationFailure> {
        if self.editor.view_state().interaction != InteractionState::Idle {
            self.editor
                .execute(context, EditorCommand::CancelInteraction)
                .map_err(ApplicationFailure::Editor)?;
        }
        Ok(())
    }
    fn outcome(
        &self,
        before: EditorViewState,
        invalidation: RenderInvalidation,
        effect: Option<ApplicationEffect>,
    ) -> ApplicationOutcome {
        let projection = self.projection();
        let state = projection.editor;
        ApplicationOutcome {
            projection,
            invalidation,
            effect,
            save: None,
            retired: None,
            changes: StateChanges {
                document: before.revision != state.revision || before.session != state.session,
                session_replaced: before.session != state.session,
                settings: before.tool != state.tool || before.brush != state.brush,
                selection: before.document.selected != state.document.selected,
                interaction: before.interaction != state.interaction,
            },
        }
    }
}
#[cfg(test)]
mod tests;
