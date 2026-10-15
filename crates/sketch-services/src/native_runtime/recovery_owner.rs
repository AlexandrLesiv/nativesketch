use super::{NativeRuntime, owner_types::*, work::*};
use crate::recovery::{CheckpointMetadata, RecoveryId};
use sketch_core::application::{routing::*, *};
impl NativeRuntime {
    pub fn register_recovery(
        &mut self,
        window: u64,
        session: u64,
        identity: RecoveryId,
        name: String,
    ) -> Result<(), RuntimeError> {
        self.check_owner()?;
        self.capacity()?;
        if name.len() > 256 || name.chars().any(char::is_control) {
            return Err(RuntimeError::Invalid);
        }
        let index = self.slot(window)?;
        if self.sessions[index].app.projection().editor.session.get() != session {
            return Err(RuntimeError::Stale);
        }
        if self.sessions[index].recovery.is_some()
            || self
                .sessions
                .iter()
                .any(|s| s.recovery.as_ref().is_some_and(|r| r.identity == identity))
        {
            return Err(RuntimeError::Invalid);
        }
        self.sessions[index].recovery = Some(RecoveryRegistration {
            identity,
            name,
            state: 1,
            generation: 0,
        });
        self.enqueue(
            OperationOwner {
                window,
                session,
                request: 0,
                grant: 0,
            },
            OperationKind::Register,
            OperationState::Queued(Box::new(Work::Register(identity))),
            0,
        )?;
        self.pump()
    }
    pub fn list_recoveries(&mut self, request: u64) -> Result<u64, RuntimeError> {
        self.check_owner()?;
        self.capacity()?;
        if self.barrier.is_some() || self.stopping {
            return Err(RuntimeError::Busy);
        }
        let id = self.enqueue(
            OperationOwner {
                window: 0,
                session: 0,
                request,
                grant: 0,
            },
            OperationKind::List,
            OperationState::Queued(Box::new(Work::List)),
            0,
        )?;
        self.pump()?;
        Ok(id)
    }
    pub fn inventory(&self, event: u64) -> Result<RecoveryInventory<'_>, RuntimeError> {
        self.check_owner()?;
        let index = self.event_operation(event)?;
        let op = &self.operations[index];
        let OperationState::Ready(value) = &op.state else {
            return Err(RuntimeError::Busy);
        };
        let WorkerValue::Inventory(listing) = value.as_ref() else {
            return Err(RuntimeError::Invalid);
        };
        Ok(RecoveryInventory {
            items: &op.items,
            listing,
        })
    }
    pub(super) fn item(&self, item: u64) -> Result<CheckpointMetadata, RuntimeError> {
        self.operations
            .iter()
            .flat_map(|op| &op.items)
            .find(|(id, _)| *id == item)
            .map(|(_, metadata)| metadata.clone())
            .ok_or(RuntimeError::Stale)
    }
    pub fn open_recovery(
        &mut self,
        window: u64,
        context: CommandContext,
        item: u64,
    ) -> Result<ActionResult, RuntimeError> {
        self.check_owner()?;
        let metadata = self.item(item)?;
        let intent = FileIntent::Navigate(NavigationIntent::Recover(RecoverySourceId(item)));
        self.file_available(window, intent)?;
        let index = self.slot(window)?;
        let outcome = self.sessions[index]
            .app
            .dispatch(
                &mut self.coordinator,
                context,
                ApplicationCommand::File(intent),
            )
            .map_err(|error| RuntimeError::Application(error.failure))?;
        self.sessions[index].requested_recovery = Some(metadata);
        self.action(window, outcome, 0, 0)
    }
    pub fn discard_recovery(&mut self, request: u64, item: u64) -> Result<u64, RuntimeError> {
        let metadata = self.item(item)?;
        self.maintenance(request, 0, Work::Discard(metadata))
    }
    pub fn retry_recovery(&mut self, request: u64, window: u64) -> Result<u64, RuntimeError> {
        if window != 0 {
            self.session(window)?;
        }
        let id = self.maintenance(request, window, Work::Retry)?;
        self.operations
            .iter_mut()
            .find(|op| op.id == id)
            .expect("issued operation")
            .kind = OperationKind::RetryRecovery;
        for slot in &mut self.sessions {
            if (window == 0 || slot.window == window)
                && let Some(recovery) = &mut slot.recovery
            {
                recovery.state = 1;
            }
        }
        Ok(id)
    }
    pub fn export_diagnostics(
        &mut self,
        request: u64,
        grant: u64,
        path: std::path::PathBuf,
    ) -> Result<u64, RuntimeError> {
        self.check_owner()?;
        self.capacity()?;
        if self.barrier.is_some() || self.stopping {
            return Err(RuntimeError::Busy);
        }
        self.new_grant(grant)?;
        let id = self.enqueue(
            OperationOwner {
                window: 0,
                session: 0,
                request,
                grant,
            },
            OperationKind::Maintenance,
            OperationState::Queued(Box::new(Work::Diagnostics(path))),
            0,
        )?;
        self.release_grant(grant)?;
        self.pump()?;
        Ok(id)
    }
    pub(super) fn maintenance(
        &mut self,
        request: u64,
        window: u64,
        work: Work,
    ) -> Result<u64, RuntimeError> {
        self.check_owner()?;
        self.capacity()?;
        if self.barrier.is_some() || self.stopping {
            return Err(RuntimeError::Busy);
        }
        let id = self.enqueue(
            OperationOwner {
                window,
                session: 0,
                request,
                grant: 0,
            },
            OperationKind::Maintenance,
            OperationState::Queued(Box::new(work)),
            0,
        )?;
        self.pump()?;
        Ok(id)
    }
}
