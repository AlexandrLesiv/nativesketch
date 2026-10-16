use super::{EditorViewState, InteractionState, ToolId};
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionId {
    ChoosePen,
    ChooseSelect,
    SetBrush,
    Select,
    BeginStroke,
    CommitStroke,
    BeginTransform,
    UpdateTransform,
    CommitTransform,
    Cancel,
    Delete,
    Undo,
    Redo,
    NewDocument,
    ReplaceDocument,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisabledReason {
    ReadOnly,
    InteractionActive,
    WrongTool,
    NothingSelected,
    HistoryEmpty,
    NoActiveStroke,
    NoActiveTransform,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Availability {
    Enabled,
    Disabled(DisabledReason),
}
impl EditorViewState {
    /// Advisory control state; execute still validates values, tokens and transitions.
    /// In particular, CommitStroke additionally requires at least one actual sample.
    pub fn availability(&self, action: ActionId) -> Availability {
        use ActionId::*;
        use Availability::*;
        use DisabledReason::*;
        let editable = !matches!(action, ChooseSelect | Select | Cancel);
        if editable && !self.capabilities.edit_document {
            return Disabled(ReadOnly);
        }
        let idle = matches!(self.interaction, InteractionState::Idle);
        if !idle
            && matches!(
                action,
                ChoosePen
                    | ChooseSelect
                    | Select
                    | BeginStroke
                    | BeginTransform
                    | Delete
                    | Undo
                    | Redo
                    | NewDocument
                    | ReplaceDocument
            )
        {
            return Disabled(InteractionActive);
        }
        if matches!(action, BeginStroke) && self.tool != ToolId::Pen
            || matches!(
                action,
                Select | BeginTransform | UpdateTransform | CommitTransform
            ) && self.tool != ToolId::Select
        {
            return Disabled(WrongTool);
        }
        if matches!(action, BeginTransform | Delete) && self.document.selected == 0 {
            return Disabled(NothingSelected);
        }
        if matches!(action, Undo) && !self.document.can_undo
            || matches!(action, Redo) && !self.document.can_redo
        {
            return Disabled(HistoryEmpty);
        }
        if action == CommitStroke && !matches!(self.interaction, InteractionState::Drawing(_)) {
            return Disabled(NoActiveStroke);
        }
        if matches!(action, UpdateTransform | CommitTransform)
            && !matches!(self.interaction, InteractionState::Transforming(_))
        {
            return Disabled(NoActiveTransform);
        }
        Enabled
    }
}
