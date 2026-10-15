use crate::{Error, Result};
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// Generation is the last published base, or zero for a phone-only document.
    Draft,
    Published,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Version<'a> {
    pub namespace_id: &'a str,
    pub document_id: &'a str,
    pub revision_id: &'a str,
    pub parent_revision_id: &'a str,
    pub content_sha256: &'a str,
    pub server_generation: u64,
    pub kind: Kind,
}

impl Version<'_> {
    pub(super) fn validate(&self) -> Result<()> {
        let id = |s: &str| !s.is_empty() && s.len() <= 256 && !s.chars().any(char::is_control);
        if !id(self.namespace_id)
            || !id(self.document_id)
            || !id(self.revision_id)
            || (!self.parent_revision_id.is_empty() && !id(self.parent_revision_id))
            || self.parent_revision_id == self.revision_id
            || self.content_sha256.len() != 64
            || !self
                .content_sha256
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
            || (self.kind == Kind::Published && self.server_generation == 0)
        {
            return Err(Error::InvalidArgument);
        }
        Ok(())
    }
}
