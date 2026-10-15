use super::{Kind, Version};
use crate::{Error, Result};
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decision {
    Accept = 0,
    Duplicate = 1,
    Stale = 2,
    Conflict = 3,
    WrongIdentity = 4,
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
