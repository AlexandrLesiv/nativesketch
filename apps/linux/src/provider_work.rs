use sketch_app::{WorkItem, WorkResult};
use sketch_core::application::storage::{StorageFailure, WriteOutcome};
use sketch_services::storage::{DestinationAccess, NoFaults, write};
use std::{fs::File, io::Read, path::PathBuf};
pub(crate) struct Job {
    pub work: WorkItem,
    pub path: Option<PathBuf>,
}
pub(crate) fn execute(job: Job) -> WorkResult {
    match job.work {
        WorkItem::PrepareNew(work) => work.prepare(),
        WorkItem::Retire(work) => work.dispose(),
        WorkItem::Read(work) => {
            let result = job
                .path
                .ok_or(std::io::ErrorKind::NotFound.into())
                .and_then(|path| {
                    let file = File::open(path)?;
                    let mut bytes = Vec::new();
                    file.take(work.maximum_bytes() as u64 + 1)
                        .read_to_end(&mut bytes)?;
                    Ok::<_, std::io::Error>(bytes)
                });
            match result {
                Ok(bytes) => work.prepare(bytes),
                Err(_) => work.fail(StorageFailure::Access),
            }
        }
        WorkItem::Write(work) => {
            let encoded = match work.encode() {
                Ok(value) => value,
                Err(result) => return result,
            };
            let outcome = match job.path.and_then(|path| DestinationAccess::new(path).ok()) {
                Some(access) => {
                    let report = write(&access, encoded.bytes(), &NoFaults);
                    if report.cleanup_failed || report.directory_sync_failed {
                        crate::diagnostics::record(format_args!(
                            "event=storage_warning operation={:?} cleanup_failed={} directory_sync_failed={}",
                            encoded.info().operation,
                            report.cleanup_failed,
                            report.directory_sync_failed
                        ));
                    }
                    report.outcome
                }
                None => WriteOutcome::Failed(StorageFailure::Access),
            };
            match encoded.finish(outcome) {
                Ok(result) => result,
                Err((error, encoded)) => {
                    crate::diagnostics::record(format_args!(
                        "event=provider_outcome_rejected error={error:?}"
                    ));
                    encoded
                        .finish(WriteOutcome::Failed(StorageFailure::Access))
                        .expect("failure outcome always accepted")
                }
            }
        }
    }
}
