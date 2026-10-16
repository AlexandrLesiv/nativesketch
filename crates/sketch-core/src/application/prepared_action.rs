use super::{CycleDirection, Editor, EditorAction, FailureKind, ToolId};
use crate::{geometry::bounds, history::Change, interaction::TransformSession};

/// Private, short-lived proof: all recoverable failures are checked before cancellation.
/// Never retained across another editor mutation or exposed through a foreign boundary.
pub(super) enum PreparedAction {
    Tool(ToolId),
    Cycle(CycleDirection),
    Delete(usize),
    Undo,
    Redo,
    Scale(TransformSession),
}
impl Editor {
    pub(super) fn prepare_action(
        &self,
        action: EditorAction,
    ) -> Result<PreparedAction, FailureKind> {
        Ok(match action {
            EditorAction::ChooseTool(tool) => PreparedAction::Tool(tool),
            EditorAction::CycleSelection(direction) => PreparedAction::Cycle(direction),
            EditorAction::DeleteSelection => {
                let index = self
                    .selection
                    .index(&self.document)
                    .ok_or(FailureKind::InvalidTransition)?;
                self.check_revision_capacity()
                    .map_err(|_| FailureKind::ResourceLimit)?;
                PreparedAction::Delete(index)
            }
            EditorAction::Undo => {
                if !self.history.can_undo() {
                    return Err(FailureKind::InvalidTransition);
                }
                PreparedAction::Undo
            }
            EditorAction::Redo => {
                if !self.history.can_redo() {
                    return Err(FailureKind::InvalidTransition);
                }
                PreparedAction::Redo
            }
            EditorAction::ScaleSelection { factor, .. } => {
                let index = self
                    .selection
                    .index(&self.document)
                    .ok_or(FailureKind::InvalidTransition)?;
                let stroke = &self.document.strokes[index];
                let bounds = bounds(stroke, stroke.transform);
                let mut prepared = TransformSession::new(index, stroke.transform);
                prepared.update(
                    &self.document,
                    factor.get(),
                    bounds.x + bounds.width / 2.0,
                    bounds.y + bounds.height / 2.0,
                )?;
                if prepared.preview != stroke.transform {
                    self.check_revision_capacity()
                        .map_err(|_| FailureKind::ResourceLimit)?;
                }
                PreparedAction::Scale(prepared)
            }
        })
    }
    pub(super) fn apply_action(&mut self, action: PreparedAction) -> Option<ToolId> {
        self.cancel();
        match action {
            PreparedAction::Tool(tool) => return Some(tool),
            PreparedAction::Cycle(direction) => {
                self.selection
                    .select_cycle(&self.document, direction.step())
                    .expect("typed direction is valid");
                return Some(ToolId::Select);
            }
            PreparedAction::Delete(index) => {
                let stroke = self.document.strokes.remove(index);
                self.selection.clear();
                self.history.record(Change::Delete(index, stroke));
            }
            PreparedAction::Undo => {
                self.history
                    .undo(&mut self.document)
                    .expect("prepared history exists");
                self.selection.reconcile(&self.document);
            }
            PreparedAction::Redo => {
                self.history
                    .redo(&mut self.document)
                    .expect("prepared history exists");
                self.selection.reconcile(&self.document);
            }
            PreparedAction::Scale(prepared) => {
                if let Some(change) = prepared.commit(&mut self.document) {
                    self.history.record(change);
                }
            }
        }
        None
    }
}
