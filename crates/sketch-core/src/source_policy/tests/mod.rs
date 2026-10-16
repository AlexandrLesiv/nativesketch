use super::*;
use crate::Error;
pub(super) const A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
pub(super) fn version<'a>(
    revision: &'a str,
    parent: &'a str,
    generation: u64,
    kind: Kind,
    hash: &'a str,
) -> Version<'a> {
    Version {
        namespace_id: "account/library",
        document_id: "drawing",
        revision_id: revision,
        parent_revision_id: parent,
        content_sha256: hash,
        server_generation: generation,
        kind,
    }
}

mod ancestry;
mod identity;
mod immutability;
