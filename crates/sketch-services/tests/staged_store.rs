mod common;
use common::*;
use sketch_core::application::storage::{Durability, WriteOutcome};
use sketch_services::storage::{self, DestinationAccess, NoFaults, Stage, StoreHooks};
use std::{fs, io};
struct Fail {
    stage: Stage,
    cleanup: bool,
}
impl StoreHooks for Fail {
    fn at(&self, stage: Stage) -> io::Result<()> {
        if stage == self.stage || self.cleanup && stage == Stage::BeforeCleanup {
            Err(io::ErrorKind::StorageFull.into())
        } else {
            Ok(())
        }
    }
}
#[test]
fn injected_pre_replace_failures_preserve_destination_and_report_cleanup_failure() {
    let temp = Temp::new();
    let path = temp.0.join("document");
    let access = DestinationAccess::new(path.clone()).unwrap();
    for stage in [
        Stage::BeforeCreate,
        Stage::AfterCreate,
        Stage::AfterWrite,
        Stage::AfterFileSync,
        Stage::BeforeReplace,
    ] {
        fs::write(&path, FIXTURE).unwrap();
        let result = storage::write(
            &access,
            b"new",
            &Fail {
                stage,
                cleanup: false,
            },
        );
        assert!(matches!(result.outcome, WriteOutcome::Failed(_)));
        assert!(!result.cleanup_failed);
        assert_eq!(fs::read(&path).unwrap(), FIXTURE);
        assert_eq!(fs::read_dir(&temp.0).unwrap().count(), 1);
    }
    let result = storage::write(
        &access,
        b"new",
        &Fail {
            stage: Stage::AfterWrite,
            cleanup: true,
        },
    );
    assert!(result.cleanup_failed);
    assert_eq!(fs::read(&path).unwrap(), FIXTURE);
    assert_eq!(fs::read_dir(&temp.0).unwrap().count(), 2);
}
#[test]
fn directory_sync_or_post_replace_failure_reports_written_with_warning() {
    let temp = Temp::new();
    let path = temp.0.join("document");
    let access = DestinationAccess::new(path.clone()).unwrap();
    for stage in [Stage::AfterReplace, Stage::BeforeDirectorySync] {
        fs::write(&path, b"old").unwrap();
        let result = storage::write(
            &access,
            FIXTURE,
            &Fail {
                stage,
                cleanup: false,
            },
        );
        assert_eq!(
            result.outcome,
            WriteOutcome::Written(Durability::DirectorySyncUnconfirmed)
        );
        assert!(result.directory_sync_failed);
        assert_eq!(fs::read(&path).unwrap(), FIXTURE);
    }
}
#[test]
fn disappeared_destination_parent_is_access_failure_without_replacement() {
    let temp = Temp::new();
    let parent = temp.0.join("provider");
    fs::create_dir(&parent).unwrap();
    let access = DestinationAccess::new(parent.join("drawing")).unwrap();
    fs::remove_dir(parent).unwrap();
    assert!(matches!(
        storage::write(&access, FIXTURE, &NoFaults).outcome,
        WriteOutcome::Failed(_)
    ));
}
#[cfg(unix)]
#[test]
fn destination_and_parent_symlinks_are_rejected_without_touching_target() {
    use std::os::unix::fs::symlink;
    let temp = Temp::new();
    let target = temp.0.join("target");
    fs::write(&target, b"preserve").unwrap();
    let link = temp.0.join("link");
    symlink(&target, &link).unwrap();
    assert!(DestinationAccess::new(link).is_err());
    let parent = temp.0.join("directory");
    fs::create_dir(&parent).unwrap();
    let alias = temp.0.join("alias");
    symlink(&parent, &alias).unwrap();
    assert!(DestinationAccess::new(alias.join("file")).is_err());
    assert_eq!(fs::read(target).unwrap(), b"preserve");
}
