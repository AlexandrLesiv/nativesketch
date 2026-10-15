use super::{DestinationAccess, Stage, StoreHooks, StoreReport};
use sketch_core::application::storage::{Durability, StorageFailure, WriteOutcome};
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::sync::atomic::{AtomicU64, Ordering};
static NEXT_TEMP: AtomicU64 = AtomicU64::new(0);
const MAX_WRITE: usize = 64 * 1024 * 1024 + 1024;
pub fn write(access: &DestinationAccess, bytes: &[u8], hooks: &dyn StoreHooks) -> StoreReport {
    let failed = |failure, cleanup_failed, stage| StoreReport {
        outcome: WriteOutcome::Failed(failure),
        cleanup_failed,
        directory_sync_failed: false,
        failed_stage: stage,
    };
    if bytes.len() > MAX_WRITE {
        return failed(StorageFailure::Capacity, false, None);
    }
    if access.validate().is_err() {
        return failed(StorageFailure::Access, false, None);
    }
    let Some(parent) = access.path().parent() else {
        return failed(StorageFailure::Access, false, None);
    };
    if let Err(error) = hooks.at(Stage::BeforeCreate) {
        return failed(
            classify(&error, StorageFailure::Write),
            false,
            Some(Stage::BeforeCreate),
        );
    }
    let mut temporary = None;
    for _ in 0..16 {
        let nonce = NEXT_TEMP.fetch_add(1, Ordering::Relaxed);
        let path = parent.join(format!(".native-sketch-{}-{nonce}.tmp", std::process::id()));
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        match options.open(&path) {
            Ok(file) => {
                temporary = Some((path, file));
                break;
            }
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => {
                return failed(
                    classify(&error, StorageFailure::Access),
                    false,
                    Some(Stage::BeforeCreate),
                );
            }
        }
    }
    let Some((temporary, mut file)) = temporary else {
        return failed(StorageFailure::Capacity, false, Some(Stage::BeforeCreate));
    };
    let mut failed_stage = Stage::AfterCreate;
    let staged = (|| {
        hooks.at(Stage::AfterCreate)?;
        failed_stage = Stage::AfterWrite;
        file.write_all(bytes)?;
        hooks.at(Stage::AfterWrite)?;
        match fs::symlink_metadata(access.path()) {
            Ok(metadata) if metadata.file_type().is_file() => {
                file.set_permissions(metadata.permissions())?
            }
            Ok(_) => return Err(io::ErrorKind::InvalidInput.into()),
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(error),
        }
        failed_stage = Stage::AfterFileSync;
        file.sync_all()?;
        hooks.at(Stage::AfterFileSync)?;
        Ok::<_, io::Error>(())
    })();
    drop(file);
    let replacement = staged.and_then(|()| {
        failed_stage = Stage::BeforeReplace;
        hooks.at(Stage::BeforeReplace)?;
        access.validate()?;
        fs::rename(&temporary, access.path())
    });
    if let Err(error) = replacement {
        let cleanup_failed = hooks
            .at(Stage::BeforeCleanup)
            .and_then(|()| fs::remove_file(&temporary))
            .is_err();
        return failed(
            classify(
                &error,
                if failed_stage == Stage::BeforeReplace {
                    StorageFailure::Replace
                } else {
                    StorageFailure::Write
                },
            ),
            cleanup_failed,
            Some(failed_stage),
        );
    }
    // Every error after rename is a written result; never report the old copy unchanged.
    let mut post_stage = Stage::AfterReplace;
    let synced = hooks
        .at(Stage::AfterReplace)
        .and_then(|()| {
            post_stage = Stage::BeforeDirectorySync;
            hooks.at(Stage::BeforeDirectorySync)
        })
        .and_then(|()| fs::File::open(parent)?.sync_all())
        .is_ok();
    StoreReport {
        outcome: WriteOutcome::Written(if synced {
            Durability::FileAndDirectorySynced
        } else {
            Durability::DirectorySyncUnconfirmed
        }),
        cleanup_failed: false,
        directory_sync_failed: !synced,
        failed_stage: (!synced).then_some(post_stage),
    }
}
fn classify(error: &io::Error, fallback: StorageFailure) -> StorageFailure {
    match error.kind() {
        io::ErrorKind::PermissionDenied => StorageFailure::Access,
        io::ErrorKind::StorageFull => StorageFailure::Capacity,
        _ => fallback,
    }
}
