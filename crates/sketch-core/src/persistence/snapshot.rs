use crate::{Document, Result};
/// Only a fully decoded, validated candidate can enter the typed replacement API.
#[derive(Debug)]
pub struct ValidatedDocument(pub(crate) Document);
impl ValidatedDocument {
    pub fn decode(bytes: &[u8]) -> Result<Self> {
        super::decode(bytes).map(Self)
    }
}
/// Explicit owned copy for queries/workers. No mutable editor/document reference escapes.
#[derive(Debug, Clone)]
pub struct DocumentSnapshot {
    pub(crate) document: Document,
    pub session: crate::application::DocumentSessionId,
    pub revision: crate::application::Revision,
}
impl DocumentSnapshot {
    pub fn document(&self) -> &Document {
        &self.document
    }
    /// Explicit worker-capable encoding; never called by a command outcome.
    pub fn encode(&self) -> Result<Vec<u8>> {
        let bytes = super::encode(&self.document);
        if bytes.len() as u64 > super::MAX_BYTES {
            return Err(crate::Error::InvalidArgument);
        }
        Ok(bytes)
    }
}
