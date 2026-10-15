use super::Editor;
/// Constant-time compatibility projection. No bounds, scene building or encoding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EditorSummary {
    pub revision: u64,
    pub count: usize,
    pub selected: u64,
    pub can_undo: bool,
    pub can_redo: bool,
    pub dirty: bool,
    pub interaction: u32,
}
impl Editor {
    pub fn summary(&self) -> EditorSummary {
        EditorSummary {
            revision: self.history.revision,
            count: self.document.strokes.len(),
            selected: self.selection.id(),
            can_undo: self.history.can_undo(),
            can_redo: self.history.can_redo(),
            dirty: self.history.dirty(),
            interaction: self.interaction.kind(),
        }
    }
    pub(crate) fn check_revision_capacity(&self) -> crate::Result<()> {
        self.history.check_capacity()
    }
}
