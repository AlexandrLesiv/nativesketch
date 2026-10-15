use super::{BrushSettings, DeviceKind, Operation, PointerId, ToolId, ValidatedDocument};
#[derive(Debug)]
pub enum EditorCommand {
    SetTool(ToolId),
    SetBrush(BrushSettings),
    SelectAt {
        x: f64,
        y: f64,
        tolerance: f64,
    },
    SelectCycle(i32),
    ClearSelection,
    BeginStroke {
        pointer: PointerId,
        device: DeviceKind,
    },
    CommitStroke(PointerId),
    BeginTransform,
    UpdateTransform {
        scale: f64,
        anchor_x: f64,
        anchor_y: f64,
    },
    CommitTransform,
    CancelInteraction,
    DeleteSelection,
    Undo,
    Redo,
    NewDocument,
    ReplaceDocument(ValidatedDocument),
}
impl EditorCommand {
    pub(super) fn operation(&self) -> Operation {
        match self {
            Self::SetTool(_) => Operation::SetTool,
            Self::SetBrush(_) => Operation::SetBrush,
            Self::SelectAt { .. } | Self::SelectCycle(_) | Self::ClearSelection => {
                Operation::Select
            }
            Self::BeginStroke { .. } => Operation::BeginStroke,
            Self::CommitStroke(_) => Operation::CommitStroke,
            Self::BeginTransform => Operation::BeginTransform,
            Self::UpdateTransform { .. } => Operation::UpdateTransform,
            Self::CommitTransform => Operation::CommitTransform,
            Self::CancelInteraction => Operation::Cancel,
            Self::DeleteSelection => Operation::Delete,
            Self::Undo => Operation::Undo,
            Self::Redo => Operation::Redo,
            Self::NewDocument => Operation::New,
            Self::ReplaceDocument(_) => Operation::Replace,
        }
    }
    pub(super) fn needs_edit(&self) -> bool {
        !matches!(
            self,
            Self::SetTool(ToolId::Select)
                | Self::SelectAt { .. }
                | Self::SelectCycle(_)
                | Self::ClearSelection
                | Self::CancelInteraction
        )
    }
    pub(super) fn transaction_bound(&self) -> bool {
        matches!(
            self,
            Self::CommitStroke(_)
                | Self::UpdateTransform { .. }
                | Self::CommitTransform
                | Self::CancelInteraction
        )
    }
}
/// Borrowed for one call, never retained; predictions remain temporary.
#[derive(Debug, Clone, Copy)]
pub struct SampleBatch<'a> {
    pub pointer: PointerId,
    pub samples: &'a [crate::Input],
}
