use super::{io_roots, io_state::IoState, work::*};
use crate::{
    recovery::DiscardProgress,
    storage::{DestinationAccess, StoreHooks, StoreReport},
    worker::{ServiceReceipt, Submission, WriteTarget},
};
use sketch_core::application::storage::WriteOutcome;
impl IoState {
    pub fn perform(&mut self, work: Work, hooks: &dyn StoreHooks) -> WorkerValue {
        match work {
            Work::Resolve(path) => self
                .resolve(path)
                .map(WorkerValue::Destination)
                .unwrap_or_else(WorkerValue::Failure),
            Work::Prepare { ticket, recovery } => {
                WorkerValue::Replacement(Box::new(self.prepare(ticket, recovery, || Ok(()))))
            }
            Work::Write { job, checkpoint } => {
                let target = if let Some(metadata) = checkpoint {
                    self.recovery
                        .as_ref()
                        .map_err(|error| *error)
                        .and_then(|access| {
                            access
                                .store
                                .ensure_allocated(metadata.identity)
                                .map_err(NativeFailure::Recovery)?;
                            Ok(WriteTarget::Checkpoint {
                                store: access.store.clone(),
                                metadata,
                            })
                        })
                } else {
                    self.destinations
                        .iter()
                        .find(|entry| entry.id == job.token().destination())
                        .ok_or(NativeFailure::Access)
                        .and_then(|entry| {
                            DestinationAccess::new(entry.path.clone())
                                .map(WriteTarget::Document)
                                .map_err(|_| NativeFailure::Access)
                        })
                };
                let receipt = match target {
                    Ok(target) => {
                        crate::worker::perform(Submission { job, target }, self.config.build, hooks)
                    }
                    Err(error) => {
                        let outcome =
                            WriteOutcome::Failed(super::io_events::storage_failure(error));
                        ServiceReceipt {
                            write: job.complete(outcome),
                            events: Vec::new(),
                            report: StoreReport {
                                outcome,
                                cleanup_failed: false,
                                directory_sync_failed: false,
                                failed_stage: None,
                            },
                        }
                    }
                };
                WorkerValue::Write(receipt)
            }
            Work::Retire(state) => {
                drop(state);
                WorkerValue::Complete
            }
            Work::ReleaseDestination(id) => {
                self.release(id);
                WorkerValue::Complete
            }
            Work::Retry => {
                if self.recovery.is_err() {
                    self.recovery = io_roots::recovery(&self.config);
                }
                if self.diagnostics.is_err() {
                    self.diagnostics = io_roots::diagnostics(&self.config);
                }
                self.recovery
                    .as_ref()
                    .map(|_| WorkerValue::Complete)
                    .unwrap_or_else(|error| WorkerValue::Failure(*error))
            }
            Work::Diagnostics(path) => self.export_diagnostics(path, hooks),
            work => {
                if matches!(work, Work::Register(_)) && self.recovery.is_err() {
                    self.recovery = io_roots::recovery(&self.config);
                }
                self.recovery_work(work)
            }
        }
    }
    fn recovery_work(&self, work: Work) -> WorkerValue {
        let store = match &self.recovery {
            Ok(access) => &access.store,
            Err(error) => return WorkerValue::Failure(*error),
        };
        let result = match work {
            Work::Register(identity) => store
                .allocate(identity)
                .and_then(|_| store.release_empty(identity))
                .map(|_| WorkerValue::Complete),
            Work::List => store.candidates().map(WorkerValue::Inventory),
            Work::Discard(metadata) => {
                let result = store.load(metadata.identity).and_then(|candidate| {
                    if candidate.envelope.metadata().generation != metadata.generation {
                        return Err(crate::recovery::RecoveryError::Conflict);
                    }
                    // At most64 recognized files in a valid scanned session,32 removed per step.
                    for _ in 0..3 {
                        if store.discard_step(metadata.identity)? == DiscardProgress::Complete {
                            return Ok(());
                        }
                    }
                    Err(crate::recovery::RecoveryError::Capacity)
                });
                result.map(|_| WorkerValue::Complete)
            }
            Work::ResolveRecovery {
                metadata,
                now_seconds,
            } => store
                .mark_resolved(metadata.identity, metadata.generation, now_seconds)
                .map(|_| WorkerValue::Complete),
            _ => unreachable!("non-recovery operation routed separately"),
        };
        result.unwrap_or_else(|error| WorkerValue::Failure(NativeFailure::Recovery(error)))
    }
}
