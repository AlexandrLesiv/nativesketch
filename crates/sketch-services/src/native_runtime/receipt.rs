use super::{NativeRuntime, owner_types::*, work::*};
use crate::{
    diagnostics::{Event, EventCode, Subsystem},
    worker::ServiceReceipt,
};
use sketch_core::application::storage::{StorageError, StorageFailure, WriteOutcome};
impl NativeRuntime {
    pub(super) fn receive_write(
        &mut self,
        index: usize,
        mut receipt: ServiceReceipt,
    ) -> WorkerValue {
        let token = receipt.write.token();
        let exact =
            matches!(self.operations[index].kind, OperationKind::Save(issued) if issued == token);
        if !exact {
            self.poison_receipt(index, WorkerValue::Write(receipt), false);
        }
        let active = self.coordinator.is_active(token);
        match self.coordinator.finish(receipt.write) {
            Ok(()) => WorkerValue::Completed(
                Some(
                    self.coordinator
                        .take_completion(token)
                        .expect("finished issued receipt"),
                ),
                receipt.report,
            ),
            Err(rejected) => {
                let correctable = active && rejected.error() == StorageError::CapabilityDenied;
                receipt.write = rejected.into_receipt();
                if !correctable {
                    self.poison_receipt(index, WorkerValue::Write(receipt), false);
                }
                // The actual worker has returned. Its invalid report cannot certify
                // persistence; settle this exact active job as uncertain, never success.
                let uncertain = WriteOutcome::Uncertain(StorageFailure::WorkerStopped);
                receipt
                    .write
                    .revise_outcome(uncertain)
                    .expect("uncertain report is valid");
                if let Err(rejected) = self.coordinator.finish(receipt.write) {
                    receipt.write = rejected.into_receipt();
                    self.poison_receipt(index, WorkerValue::Write(receipt), false);
                }
                // Preserve the original store report; only its acknowledgment is uncertain.
                let completion = self
                    .coordinator
                    .take_completion(token)
                    .expect("settled uncertain receipt");
                self.poison_receipt(
                    index,
                    WorkerValue::Completed(Some(completion), receipt.report),
                    true,
                );
            }
        }
    }
    fn poison_receipt(&mut self, index: usize, value: WorkerValue, settled: bool) -> ! {
        let op = &mut self.operations[index];
        // Reserved operation retains proof/terminal result before FFI poisoning. Detached
        // cleanup can discard a settled completion without claiming the editor is clean.
        op.state = OperationState::Ready(Box::new(value));
        let event = Event {
            build: self.config.build,
            operation_id: op.id,
            session: op.session,
            request: op.request,
            revision: match op.kind {
                OperationKind::Save(token) => token.revision().0,
                _ => 0,
            },
            subsystem: Subsystem::Storage,
            code: EventCode::NativeFailed,
            duration_us: 0,
            count: u64::from(settled),
            failure: Some(StorageFailure::WorkerStopped),
            stage: None,
            durability: None,
            cleanup_unconfirmed: !settled,
        };
        eprint!("{}", event.encode());
        panic!("native receipt violated the coordinator contract; terminal_settled={settled}");
    }
}
