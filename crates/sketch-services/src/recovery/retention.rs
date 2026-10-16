use super::inventory::{inventory, read_bounded};
use super::{
    DiscardProgress, RETENTION_SECONDS, RecoveryEnvelope, RecoveryError, RecoveryId, RecoveryStore,
};
use crate::storage::{self, DestinationAccess, NoFaults};
use sketch_core::application::storage::WriteOutcome;
#[derive(Debug, Default, PartialEq, Eq)]
pub struct RetentionReport {
    pub removed: usize,
    pub rejected_markers: usize,
}
impl RecoveryStore {
    pub fn mark_resolved(
        &self,
        id: RecoveryId,
        generation: u64,
        now_seconds: u64,
    ) -> Result<(), RecoveryError> {
        let sessions = inventory(&self.root, self.limits)?;
        let session = sessions
            .iter()
            .find(|s| s.id == id)
            .ok_or(RecoveryError::Missing)?;
        let path = session
            .files
            .iter()
            .find(|(value, _)| *value == generation)
            .map(|(_, path)| path)
            .ok_or(RecoveryError::Conflict)?;
        let envelope = RecoveryEnvelope::decode(&read_bounded(path)?)?;
        if envelope.metadata.identity != id || envelope.metadata.generation != generation {
            return Err(RecoveryError::Invalid);
        }
        let marker_path = self.root.join(id.name()).join("resolved");
        if super::resolution::read(&marker_path, id)?.is_some_and(|old| old.generation > generation)
        {
            return Err(RecoveryError::Conflict);
        }
        let bytes = super::resolution::encode(id, generation, now_seconds);
        // Existing markers remain counted until replacement; reserve the complete new stage.
        if sessions
            .iter()
            .try_fold(bytes.len() as u64, |sum, session| {
                sum.checked_add(session.bytes)
            })
            .is_none_or(|total| total > self.limits.bytes)
        {
            return Err(RecoveryError::Capacity);
        }
        let access = DestinationAccess::new(marker_path)?;
        if matches!(
            storage::write(&access, &bytes, &NoFaults).outcome,
            WriteOutcome::Written(_)
        ) {
            Ok(())
        } else {
            Err(RecoveryError::Io)
        }
    }
    /// Conservative admission-style scan. Invalid resolution never permits deletion.
    pub fn expire_resolved(&self, now_seconds: u64) -> Result<RetentionReport, RecoveryError> {
        let mut report = RetentionReport::default();
        for session in inventory(&self.root, self.limits)? {
            let marker = self.root.join(session.id.name()).join("resolved");
            let resolved = match super::resolution::read(&marker, session.id) {
                Ok(Some(marker)) => marker,
                Ok(None) => continue,
                Err(_) => {
                    report.rejected_markers += 1;
                    continue;
                }
            };
            if session
                .files
                .first()
                .is_some_and(|(latest, _)| *latest == resolved.generation)
                && now_seconds.saturating_sub(resolved.seconds) >= RETENTION_SECONDS
            {
                // Healthy sessions have at most64 entries; maintenance stays bounded.
                if self.discard_step(session.id)? == DiscardProgress::Complete
                    || self.discard_step(session.id)? == DiscardProgress::Complete
                {
                    report.removed += 1;
                }
            }
        }
        Ok(report)
    }
}
