use super::super::{storage::*, *};
use super::replacement::{PreparedReplacement, ReplacementFailure, ReplacementTicket};
/// Correlation for one issued continuation; not authentication or a filesystem grant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EffectToken {
    pub(super) session: DocumentSessionId,
    pub(super) serial: u64,
    pub(super) request: RequestId,
}
impl EffectToken {
    pub fn session(self) -> DocumentSessionId {
        self.session
    }
    pub fn serial(self) -> u64 {
        self.serial
    }
    pub fn request(self) -> RequestId {
        self.request
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecoverySourceId(pub u64);
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavigationIntent {
    New,
    Open,
    Recover(RecoverySourceId),
    Close,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileIntent {
    Save,
    SaveAs,
    ExportCopy,
    Checkpoint(DestinationId),
    Navigate(NavigationIntent),
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnsavedChoice {
    Save,
    Discard,
    Cancel,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectDecision {
    Cancel,
    Destination(DestinationId),
    Source(DestinationId),
    Unsaved(UnsavedChoice),
    Continue,
    ConfirmClose,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EffectReply {
    pub token: EffectToken,
    pub decision: EffectDecision,
}
#[derive(Debug)]
pub enum ApplicationCommand<'a> {
    Edit(EditorCommand),
    Input(SampleBatch<'a>),
    File(FileIntent),
    ResolveEffect(EffectReply),
    CompleteSave(&'a mut Option<SaveCompletion>),
    Install(&'a mut PreparedReplacement),
    ReplacementFailed(&'a mut Option<ReplacementFailure>),
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApplicationFailure {
    Editor(EditorError),
    Storage(StorageError),
    Write(StorageFailure),
    InvalidEffect,
    StaleDocument,
    Busy,
    CapabilityDenied,
    NavigationRequired,
    ResourceLimit,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ApplicationError {
    pub context: CommandContext,
    pub failure: ApplicationFailure,
}
#[derive(Debug)]
pub enum ApplicationEffect {
    ChooseDestination {
        token: EffectToken,
        purpose: SavePurpose,
    },
    ChooseSource(EffectToken),
    ConfirmUnsaved {
        token: EffectToken,
        revision: Revision,
    },
    AwaitWrites(EffectToken),
    SaveQueued(SaveToken),
    PrepareReplacement(ReplacementTicket),
    NavigationReady(EffectToken),
    CloseConfirmed,
    Cancelled,
    Failure(ApplicationFailure),
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilePhase {
    Idle,
    Destination,
    Source,
    Unsaved,
    Writes,
    Replacement,
    CloseReady,
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ApplicationProjection {
    pub editor: EditorViewState,
    pub save: SaveState,
    pub file: FilePhase,
    pub continuation: Option<EffectToken>,
    pub navigation: Option<NavigationIntent>,
    pub outstanding_saves: u32,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AppliedSave {
    pub token: SaveToken,
    pub result: SaveResult,
    pub acknowledgement: Acknowledgement,
}
#[derive(Debug)]
pub struct ApplicationOutcome {
    pub projection: ApplicationProjection,
    pub changes: StateChanges,
    pub invalidation: RenderInvalidation,
    pub effect: Option<ApplicationEffect>,
    pub save: Option<AppliedSave>,
    /// Move to a bounded retirement worker; never discard large state in an input callback.
    pub retired: Option<EditorSession>,
}
