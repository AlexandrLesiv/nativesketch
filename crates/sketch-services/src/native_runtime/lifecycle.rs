use super::{NativeRuntime, owner_types::*, work::*};
use sketch_core::application::routing::*;
impl NativeRuntime {
    pub fn detach(&mut self, window: u64, poisoned: bool) -> Result<(), RuntimeError> {
        self.check_owner()?;
        if !poisoned {
            self.capacity()?;
        }
        let index = self.slot(window)?;
        if !poisoned && !self.sessions[index].close_confirmed && !self.sessions[index].pristine {
            return Err(RuntimeError::Denied);
        }
        if self.operations.iter().any(|op| op.window == window) {
            if !poisoned {
                return Err(RuntimeError::Busy);
            }
            self.sessions[index].detached = true;
            let grant = std::mem::take(&mut self.sessions[index].bound_grant);
            self.release_grant(grant)?;
            return Ok(());
        }
        let slot = self.sessions.remove(index);
        self.release_grant(slot.bound_grant)?;
        self.retire(Retired::Session(slot.app))?;
        self.pump()
    }
    pub fn request_shutdown(&mut self) -> Result<(), RuntimeError> {
        self.check_owner()?;
        if self
            .sessions
            .iter()
            .any(|slot| !slot.detached && !slot.close_confirmed && !slot.pristine)
        {
            return Err(RuntimeError::Busy);
        }
        self.stopping = true;
        self.io.cancel_controls();
        self.shutdown_progress()
    }
    pub(super) fn shutdown_progress(&mut self) -> Result<(), RuntimeError> {
        if self.stopping
            && self.operations.is_empty()
            && self.sessions.is_empty()
            && !self.io.stopped()
        {
            self.io.stop();
        }
        if self.stopping && self.io.stopped() && !self.shutdown_notified {
            let id = self.id()?;
            self.events.push_back(NativeEvent {
                id,
                operation: 0,
                operation_kind: 211,
                window: 0,
                session: 0,
                request: 0,
                grant: 0,
                kind: EventKind::Shutdown,
                failure: None,
            });
            self.shutdown_notified = true;
        }
        Ok(())
    }
    pub fn free_ready(&self) -> Result<bool, RuntimeError> {
        self.check_owner()?;
        Ok(self.stopping
            && self.shutdown_notified
            && self.io.stopped()
            && self.sessions.is_empty()
            && self.operations.is_empty()
            && self.events.is_empty()
            && self.delivered.is_empty()
            && self.leases.is_empty()
            && self.coordinator.outstanding() == 0)
    }
    pub fn effect_token(
        &self,
        window: u64,
        session: u64,
        serial: u64,
        request: u64,
    ) -> Result<EffectToken, RuntimeError> {
        self.session(window)?
            .app
            .projection()
            .continuation
            .filter(|token| {
                token.session().get() == session
                    && token.serial() == serial
                    && token.request().0 == request
            })
            .ok_or(RuntimeError::Stale)
    }
}
