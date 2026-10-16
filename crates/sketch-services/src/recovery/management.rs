use super::{RecoveryError, RecoveryId, RecoveryStore};
use crate::storage::validate_directory;
use std::fs;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiscardProgress {
    Complete,
    MoreEntries,
}
pub(super) fn recognized(name: &str) -> bool {
    name == "resolved"
        || staging(name)
        || name.strip_suffix(".nskr").is_some_and(|number| {
            number.len() == 16
                && number
                    .bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        })
}
pub(super) fn staging(name: &str) -> bool {
    name.starts_with(".native-sketch-") && name.ends_with(".tmp")
}
impl RecoveryStore {
    /// Destructive and bounded: repeat only after explicit native Discard confirmation.
    /// Does not scan other sessions, recurse, or follow symlinks.
    pub fn discard_step(&self, id: RecoveryId) -> Result<DiscardProgress, RecoveryError> {
        let path = self.root.join(id.name());
        validate_directory(&path).map_err(|_| RecoveryError::UnsafeEntry)?;
        for entry in fs::read_dir(&path)?.take(32) {
            let entry = entry?;
            if !entry.file_type()?.is_file() || !entry.file_name().to_str().is_some_and(recognized)
            {
                return Err(RecoveryError::UnsafeEntry);
            }
            fs::remove_file(entry.path())?;
        }
        if fs::read_dir(&path)?.next().is_some() {
            Ok(DiscardProgress::MoreEntries)
        } else {
            fs::remove_dir(path)?;
            Ok(DiscardProgress::Complete)
        }
    }
    /// Serial owner only, with no active write. Bounded cleanup of its interrupted stages.
    pub fn cleanup_staging_step(&self, id: RecoveryId) -> Result<usize, RecoveryError> {
        let path = self.root.join(id.name());
        validate_directory(&path).map_err(|_| RecoveryError::UnsafeEntry)?;
        let mut removed = 0;
        for entry in fs::read_dir(path)?.take(32) {
            let entry = entry?;
            if !entry.file_type()?.is_file() {
                return Err(RecoveryError::UnsafeEntry);
            }
            if entry.file_name().to_str().is_some_and(staging) {
                fs::remove_file(entry.path())?;
                removed += 1;
            }
        }
        Ok(removed)
    }
}
