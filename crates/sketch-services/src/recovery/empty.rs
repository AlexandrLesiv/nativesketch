use super::{RecoveryError, RecoveryId, RecoveryStore};
use crate::storage::validate_directory;
use std::fs;
impl RecoveryStore {
    /// Used only for an identity previously approved by the exclusive native owner.
    /// Generation counts attempts, not successful allocation; a retry may create its directory.
    pub(crate) fn ensure_allocated(&self, identity: RecoveryId) -> Result<(), RecoveryError> {
        let path = self.root.join(identity.name());
        match fs::symlink_metadata(&path) {
            Ok(_) => validate_directory(&path).map_err(|_| RecoveryError::UnsafeEntry),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => self.allocate(identity),
            Err(error) => Err(error.into()),
        }
    }

    /// Only verified empty private directories; never remove staging, markers or checkpoints.
    /// Crash-left empty allocations carry no recoverable data and consume no retention budget.
    pub fn prune_empty(&self) -> Result<usize, RecoveryError> {
        validate_directory(&self.root).map_err(|_| RecoveryError::UnsafeEntry)?;
        let mut removed = 0;
        for entry in fs::read_dir(&self.root)?.take(20) {
            let entry = entry?;
            let Some(identity) = entry
                .file_name()
                .to_str()
                .and_then(|name| RecoveryId::parse(name).ok())
            else {
                continue;
            };
            if self.release_empty(identity)? {
                removed += 1;
            }
        }
        Ok(removed)
    }
    pub fn release_empty(&self, identity: RecoveryId) -> Result<bool, RecoveryError> {
        let path = self.root.join(identity.name());
        validate_directory(&path).map_err(|_| RecoveryError::UnsafeEntry)?;
        if fs::read_dir(&path)?.next().is_some() {
            return Ok(false);
        }
        fs::remove_dir(path)?;
        Ok(true)
    }
}
