use super::inventory::{inventory, read_bounded};
use super::{CheckpointMetadata, RecoveryEnvelope, RecoveryError, RecoveryId, RecoveryLimits};
use crate::storage::{self, DestinationAccess, StoreHooks, StoreReport, validate_directory};
use sketch_core::application::storage::WriteOutcome;
use std::{
    fs,
    path::{Path, PathBuf},
};

/// A private local root, accessed exclusively by its serial storage owner.
/// Clones are access descriptors, not permission to perform concurrent filesystem mutations.
#[derive(Clone)]
pub struct RecoveryStore {
    pub(super) root: PathBuf,
    pub(super) limits: RecoveryLimits,
}
pub struct RecoveryCandidate {
    pub envelope: RecoveryEnvelope,
    pub rejected_generations: u32,
}
pub struct RecoveryListing {
    pub candidates: Vec<CheckpointMetadata>,
    pub rejected_sessions: usize,
    pub invalid_resolution_markers: usize,
    pub rejected_generations: usize,
}
pub(super) fn private_directory(path: &Path) -> Result<(), RecoveryError> {
    let parent = path.parent().ok_or(RecoveryError::UnsafeEntry)?;
    validate_directory(parent).map_err(|_| RecoveryError::UnsafeEntry)?;
    let mut builder = fs::DirBuilder::new();
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        builder.mode(0o700);
    }
    builder.create(path)?;
    Ok(())
}
impl RecoveryStore {
    /// Initialization performs I/O: call on a service worker, not an input/UI callback.
    pub fn open(root: PathBuf, limits: RecoveryLimits) -> Result<Self, RecoveryError> {
        if limits.sessions == 0
            || limits.sessions > 20
            || limits.bytes == 0
            || limits.bytes > 256 * 1024 * 1024
        {
            return Err(RecoveryError::Capacity);
        }
        if !root.exists() {
            private_directory(&root)?;
        }
        validate_directory(&root).map_err(|_| RecoveryError::UnsafeEntry)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if fs::metadata(&root)?.permissions().mode() & 0o077 != 0 {
                return Err(RecoveryError::UnsafeEntry);
            }
        }
        let store = Self { root, limits };
        Ok(store)
    }
    /// Host-generated random identity must acquire its directory exclusively before use.
    pub fn allocate(&self, identity: RecoveryId) -> Result<(), RecoveryError> {
        if inventory(&self.root, self.limits)?.len() >= self.limits.sessions {
            return Err(RecoveryError::Capacity);
        }
        let path = self.root.join(identity.name());
        if fs::symlink_metadata(&path).is_ok() {
            return Err(RecoveryError::Conflict);
        }
        private_directory(&path)
    }
    pub fn publish(
        &self,
        envelope: RecoveryEnvelope,
        hooks: &dyn StoreHooks,
    ) -> Result<StoreReport, RecoveryError> {
        let sessions = inventory(&self.root, self.limits)?;
        let session = sessions
            .iter()
            .find(|s| s.id == envelope.metadata.identity)
            .ok_or(RecoveryError::Missing)?;
        if session
            .files
            .first()
            .is_some_and(|(generation, _)| *generation >= envelope.metadata.generation)
        {
            return Err(RecoveryError::Conflict);
        }
        let bytes = envelope.encode();
        let total = sessions
            .iter()
            .try_fold(0u64, |sum, session| sum.checked_add(session.bytes))
            .ok_or(RecoveryError::Capacity)?;
        // Includes a full staging file and old/new versioned resolution-marker allowance.
        if total
            .checked_add(bytes.len() as u64)
            .and_then(|n| n.checked_add(128))
            .is_none_or(|n| n > self.limits.bytes)
        {
            return Err(RecoveryError::Capacity);
        }
        let path = self
            .root
            .join(envelope.metadata.identity.name())
            .join(format!("{:016x}.nskr", envelope.metadata.generation));
        let access = DestinationAccess::new(path).map_err(|_| RecoveryError::UnsafeEntry)?;
        let mut report = storage::write(&access, &bytes, hooks);
        if matches!(report.outcome, WriteOutcome::Written(_)) {
            // Keep the newest two VALID generations; corrupt files never displace a valid fallback.
            let mut kept = 1;
            for (generation, path) in &session.files {
                if let Ok(previous) = read_bounded(path).and_then(|b| RecoveryEnvelope::decode(&b))
                {
                    if previous.metadata.identity != envelope.metadata.identity
                        || previous.metadata.generation != *generation
                    {
                        continue;
                    }
                    if kept < 2 {
                        kept += 1;
                    } else if fs::remove_file(path).is_err() {
                        report.cleanup_failed = true;
                    }
                }
            }
        }
        Ok(report)
    }
    pub fn load(&self, identity: RecoveryId) -> Result<RecoveryCandidate, RecoveryError> {
        validate_directory(&self.root).map_err(|_| RecoveryError::UnsafeEntry)?;
        let session = super::inventory::scan_session(self.root.join(identity.name()), identity)?;
        let mut rejected = 0;
        for (generation, path) in &session.files {
            match read_bounded(path).and_then(|bytes| RecoveryEnvelope::decode(&bytes)) {
                Ok(envelope)
                    if envelope.metadata.identity == identity
                        && envelope.metadata.generation == *generation =>
                {
                    return Ok(RecoveryCandidate {
                        envelope,
                        rejected_generations: rejected,
                    });
                }
                _ => rejected += 1,
            }
        }
        Err(RecoveryError::Invalid)
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
        Ok(RecoveryListing {
            candidates,
            rejected_sessions,
            invalid_resolution_markers,
            rejected_generations,
        })
    }
}
