mod common;
use common::*;
use sketch_core::application::storage::{Durability, SavePurpose, StorageFailure};
use sketch_services::diagnostics::{DiagnosticRing, Event, EventCode, RotatingSink, Subsystem};
use sketch_services::storage::{DestinationAccess, Stage, StoreHooks};
use sketch_services::worker::{SerialStorageWorker, Submission, WriteTarget};
use std::{fs, io, sync::Arc};
fn event() -> Event {
    Event {
        build: 1,
        session: 2,
        request: 3,
        revision: 4,
        subsystem: Subsystem::Storage,
        code: EventCode::SaveFailed,
        duration_us: 5,
        count: 1,
        failure: Some(StorageFailure::Capacity),
        stage: Some(Stage::AfterWrite),
        durability: None,
        cleanup_unconfirmed: false,
    }
}
#[test]
fn full_rate_limited_and_failing_sinks_remain_bounded_and_observable() {
    let mut ring = DiagnosticRing::new(2, 3).unwrap();
    for request in 0..5 {
        ring.record(Event { request, ..event() }, 0);
    }
    assert_eq!(ring.len(), 2);
    assert_eq!(ring.stats().dropped, 1);
    assert_eq!(ring.stats().rate_limited, 2);
    ring.drain_to(|_| Err(io::ErrorKind::StorageFull.into()));
    assert_eq!(ring.len(), 2);
    assert_eq!(ring.stats().sink_failures, 1);
    ring.drain_to(|_| panic!("injected diagnostic sink failure"));
    assert_eq!(ring.stats().sink_failures, 2);
    let mut delivered = Vec::new();
    ring.drain_to(|event| {
        delivered.push(event);
        Ok(())
    });
    assert_eq!(delivered.len(), 2);
    assert!(ring.is_empty());
    ring.record(event(), 1000);
    assert_eq!(ring.len(), 1);
    let encoded = event().encode();
    assert!(!encoded.contains("Private"));
    assert!(!encoded.contains("drawing"));
    assert!(!encoded.contains('/'));
    assert!(encoded.contains("\"failure\":3"));
    assert!(encoded.contains("\"stage\":3"));
}
#[test]
fn rotating_sink_caps_file_count_and_bytes() {
    let temp = Temp::new();
    let mut sink = RotatingSink::new(temp.0.clone(), 512, 3).unwrap();
    for _ in 0..100 {
        sink.write(event()).unwrap();
    }
    let files: Vec<_> = fs::read_dir(&temp.0).unwrap().collect();
    assert_eq!(files.len(), 3);
    for file in files {
        assert!(file.unwrap().metadata().unwrap().len() <= 512);
    }
}
#[test]
fn oversized_existing_diagnostic_files_are_rejected_without_rotation() {
    let temp = Temp::new();
    let path = temp.0.join("diagnostics.0.jsonl");
    fs::write(&path, vec![0; 513]).unwrap();
    assert!(RotatingSink::new(temp.0.clone(), 512, 3).is_err());
    assert_eq!(fs::metadata(path).unwrap().len(), 513);
}
struct FailStage(Stage);
impl StoreHooks for FailStage {
    fn at(&self, stage: Stage) -> io::Result<()> {
        if self.0 == stage {
            Err(io::ErrorKind::StorageFull.into())
        } else {
            Ok(())
        }
    }
}
#[test]
fn worker_emits_correlated_redacted_stage_failure_and_durability_codes() {
    let temp = Temp::new();
    for stage in [Stage::AfterWrite, Stage::BeforeDirectorySync] {
        let (_, _, job) = job(SavePurpose::SaveAs);
        let worker = SerialStorageWorker::with_hooks(1, Arc::new(FailStage(stage))).unwrap();
        let path = temp.0.join("Private-document-name");
        assert!(
            worker
                .try_submit(Submission {
                    job,
                    target: WriteTarget::Document(DestinationAccess::new(path).unwrap())
                })
                .is_ok()
        );
        let result = receipt(&worker);
        let logged = result.events[0];
        assert_eq!(logged.stage, Some(stage));
        if stage == Stage::AfterWrite {
            assert_eq!(logged.failure, Some(StorageFailure::Capacity));
        } else {
            assert_eq!(
                logged.durability,
                Some(Durability::DirectorySyncUnconfirmed)
            );
        }
        assert!(!logged.encode().contains("Private-document-name"));
    }
}
