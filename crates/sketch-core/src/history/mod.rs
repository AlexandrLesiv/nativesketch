use crate::{Document, Error, Result, Stroke, Transform};
#[derive(Debug, Clone)]
pub(crate) enum Change {
    Insert(usize, Stroke),
    Delete(usize, Stroke),
    Transform(usize, Transform, Transform),
}
#[derive(Debug, Clone)]
struct Entry {
    change: Change,
    before: u64,
    after: u64,
}
#[derive(Debug)]
pub(crate) struct UndoHistory {
    undo: Vec<Entry>,
    redo: Vec<Entry>,
    pub(crate) revision: u64,
    saved_revision: u64,
    next_revision: u64,
}
impl Default for UndoHistory {
    fn default() -> Self {
        Self {
            undo: Vec::new(),
            redo: Vec::new(),
            revision: 0,
            saved_revision: 0,
            next_revision: 1,
        }
    }
}
impl UndoHistory {
    pub(crate) fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }
    pub(crate) fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }
    pub(crate) fn dirty(&self) -> bool {
        self.revision != self.saved_revision
    }
    pub(crate) fn record(&mut self, change: Change) {
        let after = self.next_revision;
        self.next_revision += 1;
        self.undo.push(Entry {
            change,
            before: self.revision,
            after,
        });
        self.revision = after;
        self.redo.clear();
    }
    pub(crate) fn replacement(&self) -> Result<Self> {
        let revision = self.next_revision;
        let next_revision = revision.checked_add(1).ok_or(Error::InvalidState)?;
        Ok(Self {
            revision,
            saved_revision: revision,
            next_revision,
            ..Self::default()
        })
    }
    pub(crate) fn mark_saved(&mut self, revision: u64) -> Result<()> {
        if revision != self.revision {
            return Err(Error::InvalidState);
        }
        self.saved_revision = revision;
        Ok(())
    }
    pub(crate) fn undo(&mut self, document: &mut Document) -> Result<()> {
        let entry = self.undo.pop().ok_or(Error::InvalidState)?;
        apply(document, &entry.change, false);
        self.revision = entry.before;
        self.redo.push(entry);
        Ok(())
    }
    pub(crate) fn redo(&mut self, document: &mut Document) -> Result<()> {
        let entry = self.redo.pop().ok_or(Error::InvalidState)?;
        apply(document, &entry.change, true);
        self.revision = entry.after;
        self.undo.push(entry);
        Ok(())
    }
}
fn apply(document: &mut Document, change: &Change, forward: bool) {
    match (change, forward) {
        (Change::Insert(i, s), true) | (Change::Delete(i, s), false) => {
            document.strokes.insert(*i, s.clone())
        }
        (Change::Insert(i, _), false) | (Change::Delete(i, _), true) => {
            document.strokes.remove(*i);
        }
        (Change::Transform(i, a, b), _) => {
            document.strokes[*i].transform = if forward { *b } else { *a }
        }
    }
}
