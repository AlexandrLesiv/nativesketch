mod common;
use common::*;
use sketch_core::application::storage::{
    Acknowledgement, SavePurpose, StorageFailure, WriteOutcome,
};
use sketch_services::recovery::{RecoveryLimits, RecoveryStore};
use sketch_services::storage::{DestinationAccess, Stage, StoreHooks};
use sketch_services::worker::{SerialStorageWorker, Submission, WriteTarget};
use std::{fs, io, sync::Arc};
struct PanicAfterReplace;
impl StoreHooks for PanicAfterReplace {
    fn at(&self, stage: Stage) -> io::Result<()> {
        assert_ne!(stage, Stage::AfterReplace, "injected worker panic");
        Ok(())
    }
}
#[test]
fn native_worker_writes_owned_snapshot_and_returns_correlated_receipt() {
    let temp = Temp::new();
    let path = temp.0.join("drawing");
    let (mut session, mut queue, job) = job(SavePurpose::SaveAs);
    let token = job.token();
    let worker = SerialStorageWorker::spawn(123).unwrap();
    assert!(
        worker
            .try_submit(Submission {
                job,
                target: WriteTarget::Document(DestinationAccess::new(path.clone()).unwrap())
            })
            .is_ok()
    );
    let receipt = receipt(&worker);
    assert_eq!(receipt.events[0].request, token.request().0);
    assert_eq!(receipt.events[0].session, token.session().get());
    queue.finish(receipt.write).unwrap();
    assert_eq!(
        session.acknowledge_save(queue.take_completion(token).unwrap()),
        Acknowledgement::SavedCurrent
    );
    assert_eq!(fs::read(path).unwrap(), FIXTURE);
}
#[test]
fn worker_panic_after_replace_delivers_uncertain_terminal_receipt() {
    let temp = Temp::new();
    let path = temp.0.join("drawing");
    let (_, mut queue, job) = job(SavePurpose::SaveAs);
    let token = job.token();
    let worker = SerialStorageWorker::with_hooks(1, Arc::new(PanicAfterReplace)).unwrap();
    assert!(
        worker
            .try_submit(Submission {
                job,
                target: WriteTarget::Document(DestinationAccess::new(path.clone()).unwrap())
            })
            .is_ok()
    );
    let receipt = receipt(&worker);
    assert_eq!(
        receipt.report.outcome,
        WriteOutcome::Uncertain(StorageFailure::WorkerStopped)
    );
    queue.finish(receipt.write).unwrap();
    assert!(queue.take_completion(token).is_ok());
    assert_eq!(fs::read(path).unwrap(), FIXTURE);
}
#[test]
fn checkpoint_worker_uses_envelope_store_and_cannot_write_plain_user_document() {
    let temp = Temp::new();
    let store = RecoveryStore::open(temp.0.join("recovery"), RecoveryLimits::default()).unwrap();
    store.allocate(id(1)).unwrap();
    let (_, mut queue, job) = job(SavePurpose::Checkpoint);
    let token = job.token();
    let worker = SerialStorageWorker::spawn(1).unwrap();
    assert!(
        worker
            .try_submit(Submission {
                job,
                target: WriteTarget::Checkpoint {
                    store: store.clone(),
                    metadata: metadata(1)
                }
            })
            .is_ok()
    );
    queue.finish(receipt(&worker).write).unwrap();
    assert!(queue.take_completion(token).is_ok());
    assert_eq!(store.load(id(1)).unwrap().envelope.metadata().generation, 1);
    let (_, _, job) = common::job(SavePurpose::Checkpoint);
    let destination = temp.0.join("user-document");
    assert!(
        worker
            .try_submit(Submission {
                job,
                target: WriteTarget::Document(DestinationAccess::new(destination.clone()).unwrap())
            })
            .is_ok()
    );
    assert!(matches!(
        receipt(&worker).report.outcome,
        WriteOutcome::Failed(_)
    ));
    assert!(!destination.exists());
}
