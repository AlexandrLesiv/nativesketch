use super::selection::Selection;
use crate::geometry::bounds;
use crate::history::UndoHistory;
use crate::interaction::ActiveInteraction;
use crate::{Bounds, Document};
#[derive(Debug, Clone, Copy)]
pub struct State {
    pub count: usize,
    pub selected: u64,
    pub can_undo: bool,
    pub can_redo: bool,
    pub dirty: bool,
    pub interaction: u32,
    pub bounds: Bounds,
}
pub(super) fn project(
    document: &Document,
    interaction: &ActiveInteraction,
    selection: &Selection,
    history: &UndoHistory,
) -> State {
    let bounds = selection
        .index(document)
        .map(|i| {
            let stroke = &document.strokes[i];
            bounds(stroke, interaction.transform(i).unwrap_or(stroke.transform))
        })
        .unwrap_or_default();
    State {
        count: document.strokes.len(),
        selected: selection.id(),
        can_undo: history.can_undo(),
        can_redo: history.can_redo(),
        dirty: history.dirty(),
        interaction: interaction.kind(),
        bounds,
    }
}
