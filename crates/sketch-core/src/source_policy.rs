//! Immutable source metadata policy, independent of transport, cache, and renderer.
//!
//! The host authenticates the namespace, verifies the content hash against bytes,
//! validates the document, and correlates selection/request identity before applying
//! this decision. A duplicate retains current provenance. No clock orders revisions.
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

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decision {
    Accept = 0,
    Duplicate = 1,
    Stale = 2,
    Conflict = 3,
    WrongIdentity = 4,
}

impl Version<'_> {
    fn validate(&self) -> Result<()> {
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

/// Pure comparison; neither argument nor any document is mutated on any outcome.
/// Without a current version, selection identity remains a required host check.
/// Unknown ancestry conflicts rather than assuming a draft can be discarded.
pub fn compare(current: Option<&Version<'_>>, candidate: &Version<'_>) -> Result<Decision> {
    candidate.validate()?;
    let Some(current) = current else {
        return Ok(Decision::Accept);
    };
    current.validate()?;
    if current.namespace_id != candidate.namespace_id
        || current.document_id != candidate.document_id
    {
        return Ok(Decision::WrongIdentity);
    }
    if current.revision_id == candidate.revision_id {
        // An immutable revision cannot change its bytes or provenance.
        return if current == candidate {
            Ok(Decision::Duplicate)
        } else {
            Err(Error::InvalidArgument)
        };
    }
    if current.parent_revision_id == candidate.revision_id
        && candidate.parent_revision_id == current.revision_id
    {
        return Err(Error::InvalidArgument);
    }
    // Equal published generations cannot identify different immutable revisions,
    // even if their bytes happen to match.
    if current.kind == Kind::Published
        && candidate.kind == Kind::Published
        && current.server_generation == candidate.server_generation
    {
        return Ok(Decision::Conflict);
    }
    if current.content_sha256 == candidate.content_sha256 {
        return Ok(Decision::Duplicate);
    }
    if current.kind == Kind::Published && candidate.kind == Kind::Published {
        return Ok(if candidate.server_generation > current.server_generation {
            Decision::Accept
        } else {
            Decision::Stale
        });
    }
    if candidate.parent_revision_id == current.revision_id {
        let consistent = match (current.kind, candidate.kind) {
            (Kind::Draft | Kind::Published, Kind::Draft) => {
                candidate.server_generation == current.server_generation
            }
            (Kind::Draft, Kind::Published) => {
                candidate.server_generation > current.server_generation
            }
            _ => unreachable!(),
        };
        return Ok(if consistent {
            Decision::Accept
        } else {
            Decision::Conflict
        });
    }
    if current.parent_revision_id == candidate.revision_id {
        let consistent = match (candidate.kind, current.kind) {
            (Kind::Draft | Kind::Published, Kind::Draft) => {
                candidate.server_generation == current.server_generation
            }
            (Kind::Draft, Kind::Published) => {
                current.server_generation > candidate.server_generation
            }
            _ => unreachable!(),
        };
        return Ok(if consistent {
            Decision::Stale
        } else {
            Decision::Conflict
        });
    }
    if current.kind == Kind::Draft
        && candidate.kind == Kind::Published
        && candidate.server_generation <= current.server_generation
    {
        return Ok(Decision::Stale);
    }
    Ok(Decision::Conflict)
}

#[cfg(test)]
mod tests {
    use super::*;
    const A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    fn version<'a>(
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
    #[test]
    fn identity_and_immutable_revision_precede_hash_deduplication() {
        let a = version("r1", "", 1, Kind::Published, A);
        let mut b = a;
        assert_eq!(compare(None, &a), Ok(Decision::Accept));
        assert_eq!(compare(Some(&a), &b), Ok(Decision::Duplicate));
        b.namespace_id = "different-account/library";
        assert_eq!(compare(Some(&a), &b), Ok(Decision::WrongIdentity));
        b = a;
        b.document_id = "different-drawing";
        assert_eq!(compare(Some(&a), &b), Ok(Decision::WrongIdentity));
        b = a;
        b.content_sha256 = B;
        assert_eq!(compare(Some(&a), &b), Err(Error::InvalidArgument));
        b = a;
        b.server_generation = 2;
        assert_eq!(compare(Some(&a), &b), Err(Error::InvalidArgument));
        b = a;
        b.parent_revision_id = "different-parent";
        assert_eq!(compare(Some(&a), &b), Err(Error::InvalidArgument));
        b = a;
        b.kind = Kind::Draft;
        assert_eq!(compare(Some(&a), &b), Err(Error::InvalidArgument));
    }
    #[test]
    fn published_order_uses_generation_without_clock_or_parent_assumptions() {
        let a = version("r1", "", 1, Kind::Published, A);
        let mut b = version("r3", "r2", 3, Kind::Published, B);
        assert_eq!(compare(Some(&a), &b), Ok(Decision::Accept));
        assert_eq!(compare(Some(&b), &a), Ok(Decision::Stale));
        b.server_generation = 1;
        assert_eq!(compare(Some(&a), &b), Ok(Decision::Conflict));
        b.content_sha256 = A;
        assert_eq!(compare(Some(&a), &b), Ok(Decision::Conflict));
        b.server_generation = 3;
        assert_eq!(compare(Some(&a), &b), Ok(Decision::Duplicate));
    }
    #[test]
    fn drafts_require_known_parent_and_do_not_silently_downgrade() {
        let base = version("base", "", 7, Kind::Published, A);
        let draft = version("draft", "base", 7, Kind::Draft, B);
        let child = version("child", "draft", 7, Kind::Draft, A);
        let published = version("published", "draft", 8, Kind::Published, A);
        assert_eq!(compare(Some(&base), &draft), Ok(Decision::Accept));
        assert_eq!(compare(Some(&draft), &base), Ok(Decision::Stale));
        assert_eq!(compare(Some(&draft), &child), Ok(Decision::Accept));
        assert_eq!(compare(Some(&child), &draft), Ok(Decision::Stale));
        assert_eq!(compare(Some(&draft), &published), Ok(Decision::Accept));
        assert_eq!(compare(Some(&published), &draft), Ok(Decision::Stale));
        let unrelated = version("remote", "unknown", 8, Kind::Published, A);
        assert_eq!(compare(Some(&draft), &unrelated), Ok(Decision::Conflict));
        let branch = version("branch", "base", 7, Kind::Draft, A);
        assert_eq!(compare(Some(&draft), &branch), Ok(Decision::Conflict));
        let skipped = version("grandchild", "child", 7, Kind::Draft, A);
        assert_eq!(compare(Some(&draft), &skipped), Ok(Decision::Conflict));
        let wrong_base = Version {
            server_generation: 8,
            ..child
        };
        assert_eq!(compare(Some(&draft), &wrong_base), Ok(Decision::Conflict));
        let phone_only = version("phone", "", 0, Kind::Draft, B);
        assert_eq!(
            compare(Some(&phone_only), &unrelated),
            Ok(Decision::Conflict)
        );
        let identical = Version {
            content_sha256: B,
            ..unrelated
        };
        assert_eq!(
            compare(Some(&phone_only), &identical),
            Ok(Decision::Duplicate)
        );
        // Decisions never overwrite the caller's immutable metadata.
        assert_eq!(phone_only.kind, Kind::Draft);
        assert_eq!(phone_only.server_generation, 0);
    }
    #[test]
    fn malformed_metadata_and_cycles_are_rejected() {
        let valid = version("r1", "", 1, Kind::Published, A);
        let too_long = "x".repeat(257);
        for bad in [
            Version {
                namespace_id: "",
                ..valid
            },
            Version {
                document_id: "a\0b",
                ..valid
            },
            Version {
                revision_id: &too_long,
                ..valid
            },
            Version {
                parent_revision_id: "r1",
                ..valid
            },
            Version {
                content_sha256: "abcdef",
                ..valid
            },
            Version {
                content_sha256: "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
                ..valid
            },
            Version {
                server_generation: 0,
                ..valid
            },
        ] {
            assert_eq!(compare(None, &bad), Err(Error::InvalidArgument));
            assert_eq!(compare(Some(&bad), &valid), Err(Error::InvalidArgument));
        }
        let a = version("a", "b", 0, Kind::Draft, A);
        let b = version("b", "a", 0, Kind::Draft, B);
        assert_eq!(compare(Some(&a), &b), Err(Error::InvalidArgument));
    }
}
