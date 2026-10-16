use super::{
    selection::Selection,
    state::{self, State},
};
use crate::history::{Change, UndoHistory};
use crate::interaction::ActiveInteraction;
use crate::{Brush, Document, Error, Input, PathCommand, Result};
/// Compatibility coordinator. One exclusive owner serializes every mutable call.
/// Pure document, interaction, selection and history each have a distinct owner.
#[derive(Debug)]
pub struct Editor {
    pub(super) document: Document,
    pub(super) interaction: ActiveInteraction,
    pub(super) selection: Selection,
    pub(super) history: UndoHistory,
    pub(super) next_id: u64,
}
impl Default for Editor {
    fn default() -> Self {
        Self {
            document: Document::default(),
            interaction: ActiveInteraction::default(),
            selection: Selection::default(),
            history: UndoHistory::default(),
            next_id: 1,
        }
    }
}
impl Editor {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn document(&self) -> &Document {
        &self.document
    }
    pub fn state(&self) -> State {
        state::project(
            &self.document,
            &self.interaction,
            &self.selection,
            &self.history,
        )
    }
    pub fn begin(&mut self, pointer_id: u64, device: u32, brush: Brush) -> Result<()> {
        self.interaction.begin(
            self.next_id,
            self.document.strokes.len(),
            pointer_id,
            device,
            brush,
        )
    }
    pub fn append(&mut self, pointer: u64, inputs: &[Input]) -> Result<()> {
        self.interaction.append(pointer, inputs)
    }
    pub fn commit(&mut self, pointer: u64) -> Result<()> {
        let stroke = self.interaction.commit(pointer)?;
        self.next_id = self.next_id.checked_add(1).ok_or(Error::InvalidState)?;
        let index = self.document.strokes.len();
        self.document.strokes.push(stroke.clone());
        self.history.record(Change::Insert(index, stroke));
        Ok(())
    }
    pub fn cancel(&mut self) {
        self.interaction.cancel();
    }
    pub fn select_at(&mut self, x: f64, y: f64, tolerance: f64) -> Result<()> {
        self.interaction.idle()?;
        self.selection.select_at(&self.document, x, y, tolerance)
    }
    pub fn select_cycle(&mut self, direction: i32) -> Result<()> {
        self.interaction.idle()?;
        self.selection.select_cycle(&self.document, direction)
    }
    pub fn select_clear(&mut self) -> Result<()> {
        self.interaction.idle()?;
        self.selection.clear();
        Ok(())
    }
    pub fn delete_selected(&mut self) -> Result<()> {
        self.interaction.idle()?;
        let i = self
            .selection
            .index(&self.document)
            .ok_or(Error::InvalidState)?;
        let stroke = self.document.strokes.remove(i);
        self.selection.clear();
        self.history.record(Change::Delete(i, stroke));
        Ok(())
    }
    pub fn resize_begin(&mut self) -> Result<()> {
        self.interaction
            .resize_begin(&self.document, self.selection.index(&self.document))
    }
    pub fn resize_update(&mut self, scale: f64, x: f64, y: f64) -> Result<()> {
        self.interaction.resize_update(&self.document, scale, x, y)
    }
    pub fn resize_commit(&mut self) -> Result<()> {
        if let Some(change) = self.interaction.resize_commit(&mut self.document)? {
            self.history.record(change);
        }
        Ok(())
    }
    pub fn undo(&mut self) -> Result<()> {
        self.interaction.idle()?;
        self.history.undo(&mut self.document)?;
        self.selection.reconcile(&self.document);
        Ok(())
    }
    pub fn redo(&mut self) -> Result<()> {
        self.interaction.idle()?;
        self.history.redo(&mut self.document)?;
        self.selection.reconcile(&self.document);
        Ok(())
    }
    pub fn render(&self) -> Vec<PathCommand> {
        crate::scene::build(&self.document, &self.interaction)
    }
}
