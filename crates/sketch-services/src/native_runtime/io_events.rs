use super::{io_state::IoState, work::*};
use crate::diagnostics::{Event, EventCode, Subsystem};
use sketch_core::application::{
    routing::ApplicationFailure,
    storage::{StorageFailure, WriteOutcome},
};
impl IoState {
    pub fn root_events(&mut self) {
        let recovery = self
            .recovery
            .as_ref()
            .map(|_| WorkerValue::Complete)
            .unwrap_or_else(|e| WorkerValue::Failure(*e));
        let diagnostics = self
            .diagnostics
            .as_ref()
            .map(|_| WorkerValue::Complete)
            .unwrap_or_else(|e| WorkerValue::Failure(*e));
        self.operation_event(
            0,
            Correlation::default(),
            Subsystem::Recovery,
            12,
            Some(&recovery),
            0,
        );
        self.operation_event(
            0,
            Correlation::default(),
            Subsystem::Diagnostics,
            12,
            Some(&diagnostics),
            0,
        );
        let failures = self
            .recovery
            .as_ref()
            .ok()
            .map(|a| a.maintenance_failures)
            .unwrap_or([None; 2]);
        for failure in failures.into_iter().flatten() {
            self.operation_event(
                0,
                Correlation::default(),
                Subsystem::Recovery,
                13,
                Some(&WorkerValue::Failure(failure)),
                0,
            );
        }
    }

    pub fn operation_event(
        &mut self,
        operation: u64,
        context: Correlation,
        subsystem: Subsystem,
        kind: u64,
        result: Option<&WorkerValue>,
        duration_us: u64,
    ) {
        let failure = result.and_then(|result| match result {
            WorkerValue::Failure(failure) => Some(storage_failure(*failure)),
            WorkerValue::Replacement(result) => {
                result.as_ref().as_ref().err().map(|f| match f.failure() {
                    ApplicationFailure::Write(failure) => failure,
                    _ => StorageFailure::Encode,
                })
            }
            WorkerValue::Write(receipt) => match receipt.report.outcome {
                WriteOutcome::Failed(failure) | WriteOutcome::Uncertain(failure) => Some(failure),
                _ => None,
            },
            _ => None,
        });
        let event = Event {
            build: self.config.build,
            operation_id: operation,
            session: context.session,
            request: context.request,
            revision: context.revision,
            subsystem,
            code: if result.is_none() {
                EventCode::NativeStarted
            } else if failure.is_some() {
                EventCode::NativeFailed
            } else {
                EventCode::NativeCompleted
            },
            duration_us,
            count: kind,
            failure,
            stage: None,
            durability: None,
            cleanup_unconfirmed: false,
        };
        self.ring
            .record(event, self.started.elapsed().as_millis() as u64);
        if let Some(WorkerValue::Write(receipt)) = result {
            for event in &receipt.events {
                self.ring.record(
                    Event {
                        operation_id: operation,
                        ..*event
                    },
                    self.started.elapsed().as_millis() as u64,
                );
            }
        }
    }
}
pub(super) fn classify(work: &Work) -> (Subsystem, u64) {
    match work {
        Work::Resolve(_) => (Subsystem::Storage, 1),
        Work::Prepare {
            recovery: Some(_), ..
        } => (Subsystem::Recovery, 2),
        Work::Prepare { .. } => (Subsystem::Storage, 2),
        Work::Write {
            checkpoint: Some(_),
            ..
        } => (Subsystem::Recovery, 3),
        Work::Write { .. } => (Subsystem::Storage, 3),
        Work::Register(_) => (Subsystem::Recovery, 4),
        Work::List => (Subsystem::Recovery, 5),
        Work::Discard(_) => (Subsystem::Recovery, 6),
        Work::ResolveRecovery { .. } => (Subsystem::Recovery, 7),
        Work::Retry => (Subsystem::Recovery, 8),
        Work::Diagnostics(_) => (Subsystem::Diagnostics, 9),
        Work::Retire(_) | Work::ReleaseDestination(_) => (Subsystem::Storage, 10),
        #[cfg(feature = "native-test-controls")]
        Work::ArmTest(_) => (Subsystem::Storage, 11),
    }
}
pub(super) fn storage_failure(failure: NativeFailure) -> StorageFailure {
    match failure {
        NativeFailure::Capacity
        | NativeFailure::Recovery(crate::recovery::RecoveryError::Capacity) => {
            StorageFailure::Capacity
        }
        NativeFailure::Corrupt
        | NativeFailure::Recovery(crate::recovery::RecoveryError::Invalid) => {
            StorageFailure::Encode
        }
        NativeFailure::Stopped | NativeFailure::Panic => StorageFailure::WorkerStopped,
        _ => StorageFailure::Access,
    }
}
