use super::{NativeRuntime, owner_types::*};
use crate::recovery::{CheckpointMetadata, ScheduledCheckpoint};
use sketch_core::application::{routing::*, storage::*, *};
impl NativeRuntime {
    pub fn idle(&mut self) -> Result<(), RuntimeError> {
        self.check_owner()?;
        if self.stopping || self.barrier.is_some() || self.capacity().is_err() {
            return Ok(());
        }
        let now = self.started.elapsed().as_millis() as u64;
        for index in 0..self.sessions.len() {
            let slot = &mut self.sessions[index];
            if slot.detached
                || !slot
                    .recovery
                    .as_ref()
                    .is_some_and(|recovery| recovery.state == 2 && recovery.approved)
            {
                continue;
            }
            let state = slot.app.projection();
            let Some(scheduled) = slot.schedule.due(
                now,
                state.editor.interaction != InteractionState::Idle,
                self.coordinator.explicit_outstanding(state.editor.session) != 0,
            ) else {
                continue;
            };
            let recovery = slot.recovery.as_mut().expect("checked registration");
            recovery.generation = recovery
                .generation
                .checked_add(1)
                .ok_or(RuntimeError::Capacity)?;
            let metadata = CheckpointMetadata {
                identity: recovery.identity,
                generation: recovery.generation,
                revision: scheduled.revision,
                created_seconds: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map_err(|_| RuntimeError::Invalid)?
                    .as_secs(),
                display_name: recovery.name.clone(),
            };
            let window = slot.window;
            let context = CommandContext {
                session: state.editor.session,
                request: RequestId(0),
                interaction: None,
            };
            let outcome = slot
                .app
                .dispatch(
                    &mut self.coordinator,
                    context,
                    ApplicationCommand::File(FileIntent::Checkpoint(DestinationId(
                        u64::MAX - window,
                    ))),
                )
                .map_err(|error| RuntimeError::Application(error.failure))?;
            let result = self.action(window, outcome, 0, 0)?;
            if let Some(operation) = self
                .operations
                .iter_mut()
                .find(|op| op.id == result.operation)
            {
                operation.checkpoint = Some(metadata);
            } else {
                self.sessions[index].schedule.complete(scheduled, true);
                self.sessions[index].recovery.as_mut().unwrap().state = 3;
            }
            break;
        }
        self.pump()
    }
    pub(super) fn checkpoint_complete(
        &mut self,
        window: u64,
        metadata: Option<&CheckpointMetadata>,
        save: &AppliedSave,
    ) -> Result<(), RuntimeError> {
        let Some(metadata) = metadata else {
            return Ok(());
        };
        let index = self.slot(window)?;
        let failed = !matches!(save.result, SaveResult::Written(_));
        let slot = &mut self.sessions[index];
        slot.schedule.complete(
            ScheduledCheckpoint {
                session: save.token.session().get(),
                revision: save.token.revision().0,
            },
            failed,
        );
        if slot.app.projection().editor.session == save.token.session() {
            if failed {
                if let Some(recovery) = &mut slot.recovery {
                    recovery.state = 3;
                }
            } else {
                slot.last_checkpoint = Some(metadata.clone());
            }
        }
        Ok(())
    }
}
