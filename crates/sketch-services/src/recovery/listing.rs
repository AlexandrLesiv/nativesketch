use super::{CheckpointMetadata, RecoveryError, RecoveryId, RecoveryStore};
use crate::storage::validate_directory;
use std::fs;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecoveryUsage {
    pub sessions: usize,
    pub bytes: u64,
}
pub struct RecoveryListing {
    pub usage: Option<RecoveryUsage>,
    pub capacity_blocked: bool,
    pub limit_bytes: u64,
    pub candidates: Vec<CheckpointMetadata>,
    pub rejected_sessions: usize,
    pub invalid_resolution_markers: usize,
    pub rejected_generations: usize,
}

impl RecoveryStore {
    fn usage(&self) -> Result<RecoveryUsage, RecoveryError> {
        let sessions = super::inventory::inventory(&self.root, self.limits)?;
        let bytes = sessions
            .iter()
            .try_fold(0u64, |sum, session| sum.checked_add(session.bytes))
            .ok_or(RecoveryError::Capacity)?;
        Ok(RecoveryUsage {
            sessions: sessions.len(),
            bytes,
        })
    }
    pub fn candidates(&self) -> Result<RecoveryListing, RecoveryError> {
        validate_directory(&self.root).map_err(|_| RecoveryError::UnsafeEntry)?;
        let mut candidates = Vec::new();
        let mut rejected_sessions = 0;
        let mut invalid_resolution_markers = 0;
        let mut rejected_generations = 0;
        for (count, entry) in fs::read_dir(&self.root)?.enumerate() {
            if count >= 20 {
                rejected_sessions += 1;
                break;
            }
            let entry = entry?;
            let identity = entry
                .file_name()
                .to_str()
                .and_then(|name| RecoveryId::parse(name).ok());
            let Some(identity) = identity else {
                rejected_sessions += 1;
                continue;
            };
            match self.load(identity) {
                Ok(candidate) => {
                    rejected_generations += candidate.rejected_generations as usize;
                    match super::resolution::read(&entry.path().join("resolved"), identity) {
                        Ok(Some(marker))
                            if marker.generation == candidate.envelope.metadata.generation => {}
                        Err(_) => {
                            invalid_resolution_markers += 1;
                            candidates.push(candidate.envelope.metadata);
                        }
                        _ => candidates.push(candidate.envelope.metadata),
                    }
                }
                Err(_) => rejected_sessions += 1,
            }
        }
        let usage = self.usage();
        let capacity_blocked = match &usage {
            Ok(usage) => {
                usage.sessions >= self.limits.sessions
                    || usage
                        .bytes
                        .checked_add(128)
                        .is_none_or(|bytes| bytes >= self.limits.bytes)
            }
            Err(RecoveryError::Capacity) => true,
            Err(_) => false,
        };
        Ok(RecoveryListing {
            usage: usage.ok(),
            capacity_blocked,
            limit_bytes: self.limits.bytes,
            candidates,
            rejected_sessions,
            invalid_resolution_markers,
            rejected_generations,
        })
    }
}
