use super::{NativeRuntime, owner_types::*, work::*};
use sketch_core::application::{
    routing::AppliedSave,
    storage::{SavePurpose, SaveResult},
};
impl NativeRuntime {
    pub(super) fn registration_result(&mut self, index: usize, value: &WorkerValue) {
        let op = &self.operations[index];
        if !matches!(
            op.kind,
            OperationKind::Register(_) | OperationKind::RetryRecovery(_)
        ) {
            return;
        }
        for slot in &mut self.sessions {
            if (op.window == 0 || op.window == slot.window)
                && (op.session == 0 || op.session == slot.app.projection().editor.session.get())
                && let Some(recovery) = &mut slot.recovery
            {
                let matches_identity = match op.kind {
                    OperationKind::Register(identity) => {
                        op.session != 0 && identity == recovery.identity
                    }
                    OperationKind::RetryRecovery(Some(identity)) => {
                        op.session != 0 && identity == recovery.identity
                    }
                    OperationKind::RetryRecovery(None) => op.window == 0 && recovery.approved,
                    _ => false,
                };
                if !matches_identity {
                    continue;
                }
                recovery.state = match value {
                    WorkerValue::Complete => {
                        if matches!(op.kind, OperationKind::Register(_)) {
                            recovery.approved = true;
                        }
                        if recovery.approved {
                            slot.schedule.resume();
                            2
                        } else {
                            5
                        }
                    }
                    WorkerValue::Failure(NativeFailure::Recovery(
                        crate::recovery::RecoveryError::Capacity,
                    )) => 4,
                    _ => 5,
                };
            }
        }
    }
    pub(super) fn saved_recovery(
        &mut self,
        window: u64,
        save: &AppliedSave,
    ) -> Result<(), RuntimeError> {
        if !matches!(
            save.token.purpose(),
            SavePurpose::Save | SavePurpose::SaveAs
        ) || !matches!(save.result, SaveResult::Written(_))
        {
            return Ok(());
        }
        let index = self.slot(window)?;
        if self.sessions[index].app.projection().editor.session != save.token.session() {
            return Ok(());
        }
        let source = self.sessions[index].recovered_from.take();
        let checkpoint = self.sessions[index]
            .last_checkpoint
            .as_ref()
            .filter(|metadata| metadata.revision == save.token.revision().0)
            .cloned();
        let now_seconds = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|_| RuntimeError::Invalid)?
            .as_secs();
        for metadata in source.into_iter().chain(checkpoint) {
            self.enqueue(
                OperationOwner {
                    window,
                    session: save.token.session().get(),
                    request: save.token.request().0,
                    grant: 0,
                },
                OperationKind::Maintenance,
                OperationState::Queued(Box::new(Work::ResolveRecovery {
                    metadata,
                    now_seconds,
                })),
                0,
            )?;
        }
        Ok(())
    }
}
