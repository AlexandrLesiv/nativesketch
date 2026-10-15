use super::{
    BrushSettings, Capabilities, DocumentSessionId, EditorSummary, InteractionId, Revision,
    SceneGeneration, ToolId,
};
pub use crate::persistence::snapshot::{DocumentSnapshot, ValidatedDocument};
use crate::{Bounds, PathCommand};
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InteractionState {
    Idle,
    Drawing(InteractionId),
    Transforming(InteractionId),
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EditorViewState {
    pub session: DocumentSessionId,
    pub revision: Revision,
    pub scene_generation: SceneGeneration,
    pub tool: ToolId,
    pub brush: BrushSettings,
    pub capabilities: Capabilities,
    pub interaction: InteractionState,
    pub document: EditorSummary,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderInvalidation {
    None,
    Overlay,
    Full,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StateChanges {
    pub document: bool,
    pub session_replaced: bool,
    pub settings: bool,
    pub selection: bool,
    pub interaction: bool,
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CommandOutcome {
    pub state: EditorViewState,
    pub changes: StateChanges,
    pub invalidation: RenderInvalidation,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorQuery {
    ViewState,
    SelectedBounds,
    Scene,
    CommittedSnapshot,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SceneKey {
    pub session: DocumentSessionId,
    pub revision: Revision,
    pub generation: SceneGeneration,
}
#[derive(Debug)]
pub struct SceneSnapshot {
    pub key: SceneKey,
    pub commands: Vec<PathCommand>,
}
#[derive(Debug)]
pub enum QueryResult {
    ViewState(EditorViewState),
    SelectedBounds(Bounds),
    Scene(SceneSnapshot),
    Document(DocumentSnapshot),
}
