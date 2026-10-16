use super::{Availability, DisabledReason, EditorViewState, FailureKind, Operation, ToolId};

/// UI and future agent intents; primitive gesture commands remain separate.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EditorAction {
    ChooseTool(ToolId),
    CycleSelection(CycleDirection),
    DeleteSelection,
    Undo,
    Redo,
    ScaleSelection {
        factor: ScaleFactor,
        anchor: ScaleAnchor,
    },
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CycleDirection {
    Previous,
    Next,
}
impl CycleDirection {
    pub(super) fn step(self) -> i32 {
        match self {
            Self::Previous => -1,
            Self::Next => 1,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScaleAnchor {
    SelectionCenter,
}
/// Relative scale; preserves the existing 10000% application upper bound.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScaleFactor(f64);
impl ScaleFactor {
    pub fn get(self) -> f64 {
        self.0
    }
}
impl TryFrom<f64> for ScaleFactor {
    type Error = FailureKind;
    fn try_from(value: f64) -> Result<Self, Self::Error> {
        if value.is_finite() && value > 0.0 && value <= 100.0 {
            Ok(Self(value))
        } else {
            Err(FailureKind::InvalidValue)
        }
    }
}
impl EditorAction {
    pub(super) fn operation(self) -> Operation {
        match self {
            Self::ChooseTool(_) => Operation::SetTool,
            Self::CycleSelection(_) => Operation::Select,
            Self::DeleteSelection => Operation::Delete,
            Self::Undo => Operation::Undo,
            Self::Redo => Operation::Redo,
            Self::ScaleSelection { .. } => Operation::CommitTransform,
        }
    }
}
impl EditorViewState {
    /// Advisory committed-state availability, ignoring the interaction that a valid
    /// action will cancel. Execute additionally validates context, geometry and capacity.
    pub fn action_availability(&self, action: EditorAction) -> Availability {
        use Availability::*;
        use DisabledReason::*;
        use EditorAction::*;
        if !self.capabilities.edit_document
            && !matches!(action, ChooseTool(ToolId::Select) | CycleSelection(_))
        {
            return Disabled(ReadOnly);
        }
        if matches!(action, CycleSelection(_)) && self.document.count == 0 {
            return Disabled(NothingSelected);
        }
        if matches!(action, DeleteSelection | ScaleSelection { .. }) && self.document.selected == 0
        {
            return Disabled(NothingSelected);
        }
        if matches!(action, Undo) && !self.document.can_undo
            || matches!(action, Redo) && !self.document.can_redo
        {
            return Disabled(HistoryEmpty);
        }
        Enabled
    }
}
