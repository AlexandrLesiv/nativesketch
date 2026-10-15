use super::Editor;
use crate::{Document, Error, Result, persistence};
use std::path::Path;
/// Synchronous compatibility persistence; worker effects are introduced separately.
impl Editor {
    pub fn clear(&mut self) -> Result<()> {
        self.interaction.idle()?;
        self.replace_document(Document::default())
    }
    /// Immutable committed document bytes and editor-local revision identity.
    pub fn export(&self) -> Result<(Vec<u8>, u64)> {
        self.interaction.idle()?;
        let bytes = persistence::encode(&self.document);
        if bytes.len() as u64 > persistence::MAX_BYTES {
            return Err(Error::InvalidArgument);
        }
        Ok((bytes, self.history.revision))
    }
    /// Replace only after full validation. Failed or cancelled host reads do not mutate state.
    pub fn import(&mut self, bytes: &[u8]) -> Result<()> {
        self.interaction.idle()?;
        self.replace_document(persistence::decode(bytes)?)
    }
    /// Host calls only after successful durable write of this editor's snapshot.
    pub fn mark_saved(&mut self, revision: u64) -> Result<()> {
        self.interaction.idle()?;
        if revision != self.history.revision {
            return Err(Error::InvalidState);
        }
        self.history.mark_saved(revision)?;
        Ok(())
    }
    pub fn save(&mut self, path: &Path) -> Result<()> {
        self.interaction.idle()?;
        persistence::save(&self.document, path)?;
        self.history.mark_saved(self.history.revision)?;
        Ok(())
    }
    pub fn load(&mut self, path: &Path) -> Result<()> {
        self.interaction.idle()?;
        let document = persistence::load(path)?;
        self.replace_document(document)
    }
    pub(crate) fn replace_document(&mut self, document: Document) -> Result<()> {
        let history = self.history.replacement()?;
        let next_id = document
            .strokes
            .iter()
            .map(|s| s.id)
            .max()
            .unwrap_or(0)
            .checked_add(1)
            .ok_or(Error::Format)?;
        *self = Self {
            document,
            next_id,
            history,
            ..Self::new()
        };
        Ok(())
    }
}
