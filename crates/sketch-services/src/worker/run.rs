use super::{ServiceReceipt, Submission, WriteTarget};
use crate::diagnostics::{Event, EventCode, Subsystem};
use crate::recovery::{RecoveryEnvelope, RecoveryError};
use crate::storage::{self, StoreHooks, StoreReport};
use sketch_core::application::storage::{SavePurpose, StorageFailure, WriteOutcome};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc::{Receiver, RecvTimeoutError, SyncSender},
};
use std::time::{Duration, Instant};
pub(super) fn worker(
    input: Receiver<Box<Submission>>,
    output: SyncSender<ServiceReceipt>,
    stop: &AtomicBool,
    build: u64,
    hooks: &dyn StoreHooks,
) {
    loop {
        let submission = match input.recv_timeout(Duration::from_millis(25)) {
            Ok(job) => job,
            Err(RecvTimeoutError::Disconnected) => break,
            Err(RecvTimeoutError::Timeout) if stop.load(Ordering::Acquire) => break,
            Err(RecvTimeoutError::Timeout) => continue,
        };
        let receipt = perform(*submission, build, hooks);
        // Backpressure stays on this worker; a closed receiver ends the worker safely.
        if output.send(receipt).is_err() {
            break;
        }
    }
}
fn perform(submission: Submission, build: u64, hooks: &dyn StoreHooks) -> ServiceReceipt {
    let started = Instant::now();
    let token = submission.job.token();
    // Keep the unique job outside the unwind boundary so panic still yields its receipt.
    let report = catch_unwind(AssertUnwindSafe(|| match submission.job.encode() {
        Ok(bytes) => match (&submission.target, token.purpose()) {
            (WriteTarget::Document(destination), purpose) if purpose != SavePurpose::Checkpoint => {
                storage::write(destination, &bytes, hooks)
            }
            (WriteTarget::Checkpoint { store, metadata }, SavePurpose::Checkpoint)
                if metadata.revision == token.revision().0 =>
            {
                RecoveryEnvelope::new(metadata.clone(), bytes)
                    .and_then(|envelope| store.publish(envelope, hooks))
                    .unwrap_or_else(|error| {
                        failed(match error {
                            RecoveryError::Capacity => StorageFailure::Capacity,
                            RecoveryError::Invalid => StorageFailure::Encode,
                            _ => StorageFailure::Access,
                        })
                    })
            }
            _ => failed(StorageFailure::Access),
        },
        Err(_) => StoreReport {
            outcome: WriteOutcome::Failed(StorageFailure::Encode),
            cleanup_failed: false,
            directory_sync_failed: false,
            failed_stage: None,
        },
    }))
    .unwrap_or(StoreReport {
        outcome: WriteOutcome::Uncertain(StorageFailure::WorkerStopped),
        cleanup_failed: true,
        directory_sync_failed: false,
        failed_stage: None,
    });
    let code = match report.outcome {
        WriteOutcome::Written(_) if token.purpose() == SavePurpose::Checkpoint => {
            EventCode::CheckpointPublished
        }
        WriteOutcome::Written(_) => EventCode::SaveWritten,
        WriteOutcome::Failed(_) => EventCode::SaveFailed,
        WriteOutcome::Uncertain(_) => EventCode::SaveUncertain,
    };
    let event = Event {
        build,
        session: token.session().get(),
        request: token.request().0,
        revision: token.revision().0,
        subsystem: Subsystem::Storage,
        code,
        duration_us: started.elapsed().as_micros().min(u64::MAX as u128) as u64,
        count: 1,
        failure: match report.outcome {
            WriteOutcome::Failed(f) | WriteOutcome::Uncertain(f) => Some(f),
            _ => None,
        },
        durability: match report.outcome {
            WriteOutcome::Written(d) => Some(d),
            _ => None,
        },
        stage: report.failed_stage,
        cleanup_unconfirmed: report.cleanup_failed,
    };
    let mut events = vec![event];
    if report.cleanup_failed {
        events.push(Event {
            code: EventCode::CleanupFailed,
            ..event
        });
    }
    ServiceReceipt {
        write: submission.job.complete(report.outcome),
        report,
        events,
    }
}
fn failed(failure: StorageFailure) -> StoreReport {
    StoreReport {
        outcome: WriteOutcome::Failed(failure),
        cleanup_failed: false,
        directory_sync_failed: false,
        failed_stage: None,
    }
}
