use super::ids::CommandContext;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operation {
    Construct,
    SetTool,
    SetBrush,
    Select,
    BeginStroke,
    Append,
    CommitStroke,
    BeginTransform,
    UpdateTransform,
    CommitTransform,
    Cancel,
    Delete,
    Undo,
    Redo,
    New,
    Replace,
    Query,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FailureKind {
    InvalidValue,
    InvalidTransition,
    ReadOnly,
    StaleSession,
    StaleInteraction,
    ResourceLimit,
    CorruptDocument,
    ServiceFailure,
}
/// Structured redacted context: no samples, bytes, document names or filesystem paths.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EditorError {
    pub operation: Operation,
    pub kind: FailureKind,
    pub context: Option<CommandContext>,
}
impl From<crate::Error> for FailureKind {
    fn from(error: crate::Error) -> Self {
        match error {
            crate::Error::InvalidArgument => Self::InvalidValue,
            crate::Error::InvalidState => Self::InvalidTransition,
            crate::Error::Format => Self::CorruptDocument,
            crate::Error::Io => Self::ServiceFailure,
        }
    }
}
