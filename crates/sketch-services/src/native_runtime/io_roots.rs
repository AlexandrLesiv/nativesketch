use super::{RuntimeConfig, work::NativeFailure};
use crate::diagnostics::RotatingSink;
use crate::recovery::{RecoveryLimits, RecoveryStore};
use std::{
    fs::{File, OpenOptions},
    path::Path,
};
pub(super) struct DiagnosticAccess {
    pub sink: RotatingSink,
    _lock: File,
}
pub(super) struct RecoveryAccess {
    pub store: RecoveryStore,
    pub maintenance_failures: [Option<NativeFailure>; 2],
    _lock: File,
}
pub(super) fn recovery(config: &RuntimeConfig) -> Result<RecoveryAccess, NativeFailure> {
    let root = config
        .recovery_root
        .as_ref()
        .ok_or(NativeFailure::Unsupported)?;
    super::private_root::provision(root)?;
    let lock = lock_root(root)?;
    let store = RecoveryStore::open(root.join("sessions"), RecoveryLimits::default())
        .map_err(NativeFailure::Recovery)?;
    let prune_failure = store.prune_empty().err().map(NativeFailure::Recovery);
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|_| NativeFailure::Access)?
        .as_secs();
    let expiry_failure = store
        .expire_resolved(now)
        .err()
        .map(NativeFailure::Recovery);
    Ok(RecoveryAccess {
        store,
        maintenance_failures: [prune_failure, expiry_failure],
        _lock: lock,
    })
}
pub(super) fn diagnostics(config: &RuntimeConfig) -> Result<DiagnosticAccess, NativeFailure> {
    let root = config
        .diagnostics_root
        .as_ref()
        .ok_or(NativeFailure::Unsupported)?;
    super::private_root::provision(root)?;
    let lock = lock_root(root)?;
    let sink = RotatingSink::new(root.clone(), 256 * 1024, 4).map_err(|_| NativeFailure::Sink)?;
    Ok(DiagnosticAccess { sink, _lock: lock })
}

fn lock_root(root: &Path) -> Result<File, NativeFailure> {
    let path = root.join("owner.lock");
    crate::storage::DestinationAccess::new(path.clone()).map_err(|_| NativeFailure::Access)?;
    let mut options = OpenOptions::new();
    options.create(true).read(true).write(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let lock = options.open(path).map_err(|_| NativeFailure::Access)?;
    lock.try_lock()
        .map_err(|_| NativeFailure::Recovery(crate::recovery::RecoveryError::Conflict))?;
    Ok(lock)
}
