mod common;
use common::*;
use sketch_services::recovery::{RecoveryEnvelope, RecoveryLimits, RecoveryStore};
use sketch_services::storage::{self, DestinationAccess, NoFaults, Stage, StoreHooks};
use std::{
    fs, io,
    path::PathBuf,
    process::{Command, Stdio},
    time::{Duration, Instant},
};
struct StopAt {
    stage: Stage,
    ready: PathBuf,
}
impl StoreHooks for StopAt {
    fn at(&self, stage: Stage) -> io::Result<()> {
        if stage == self.stage {
            fs::write(&self.ready, b"ready")?;
            loop {
                std::thread::park_timeout(Duration::from_secs(1));
            }
        }
        Ok(())
    }
}
#[test]
#[ignore = "subprocess fixture, executed by the parent process-kill tests"]
fn child_storage_writer() {
    let root = PathBuf::from(std::env::var_os("SKETCH_KILL_FIXTURE").expect("child fixture path"));
    let stage = match std::env::var("SKETCH_KILL_STAGE").unwrap().as_str() {
        "write" => Stage::AfterWrite,
        "sync" => Stage::AfterFileSync,
        "replace" => Stage::BeforeReplace,
        "published" => Stage::AfterReplace,
        _ => panic!("invalid fixture stage"),
    };
    let hooks = StopAt {
        stage,
        ready: root.join("ready"),
    };
    let bytes = fs::read(root.join("new.nsketch")).unwrap();
    if std::env::var("SKETCH_KILL_MODE").unwrap() == "recovery" {
        let store = RecoveryStore::open(root.join("recovery"), RecoveryLimits::default()).unwrap();
        store
            .publish(RecoveryEnvelope::new(metadata(2), bytes).unwrap(), &hooks)
            .unwrap();
    } else {
        storage::write(
            &DestinationAccess::new(root.join("document.nsketch")).unwrap(),
            &bytes,
            &hooks,
        );
    }
    panic!("child did not reach its expected checkpoint");
}
fn kill_at(temp: &Temp, stage: &str, mode: &str) {
    let mut child = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "child_storage_writer",
            "--ignored",
            "--nocapture",
        ])
        .env("SKETCH_KILL_FIXTURE", &temp.0)
        .env("SKETCH_KILL_STAGE", stage)
        .env("SKETCH_KILL_MODE", mode)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    while !temp.0.join("ready").exists() && Instant::now() < deadline {
        if child.try_wait().unwrap().is_some() {
            panic!("child exited before kill checkpoint");
        }
        std::thread::sleep(Duration::from_millis(2));
    }
    let ready = temp.0.join("ready").exists();
    child.kill().unwrap();
    assert!(!child.wait().unwrap().success());
    assert!(ready, "child did not reach checkpoint before timeout");
}
#[test]
fn process_death_preserves_old_or_new_valid_document_at_publication_boundaries() {
    let new = sketch_core::Editor::new().export().unwrap().0;
    for stage in ["write", "sync", "replace", "published"] {
        let temp = Temp::new();
        fs::write(temp.0.join("document.nsketch"), FIXTURE).unwrap();
        fs::write(temp.0.join("new.nsketch"), &new).unwrap();
        kill_at(&temp, stage, "document");
        let actual = fs::read(temp.0.join("document.nsketch")).unwrap();
        assert_eq!(
            actual,
            if stage == "published" {
                new.clone()
            } else {
                FIXTURE.to_vec()
            }
        );
        sketch_core::application::ValidatedDocument::decode(&actual).unwrap();
    }
}
#[test]
fn killed_checkpoint_keeps_last_valid_generation_discoverable() {
    for stage in ["write", "published"] {
        let temp = Temp::new();
        let store =
            RecoveryStore::open(temp.0.join("recovery"), RecoveryLimits::default()).unwrap();
        store.allocate(id(1)).unwrap();
        store.publish(envelope(1), &NoFaults).unwrap();
        fs::write(
            temp.0.join("new.nsketch"),
            sketch_core::Editor::new().export().unwrap().0,
        )
        .unwrap();
        kill_at(&temp, stage, "recovery");
        let expected = if stage == "published" { 2 } else { 1 };
        assert_eq!(
            store.load(id(1)).unwrap().envelope.metadata().generation,
            expected
        );
        assert_eq!(
            store.candidates().unwrap().candidates[0].generation,
            expected
        );
        assert_eq!(store.expire_resolved(u64::MAX).unwrap().removed, 0);
        assert!(store.cleanup_staging_step(id(1)).unwrap() <= 1);
    }
}
