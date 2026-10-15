use super::{NativeRuntime, owner_types::*, work::*};
use sketch_core::application::routing::*;
impl NativeRuntime {
    pub fn release_event(&mut self, event: u64) -> Result<(), RuntimeError> {
        self.check_owner()?;
        if !self.delivered.iter().any(|item| item.id == event) {
            return Err(RuntimeError::Stale);
        }
        let Some(index) = self
            .operations
            .iter()
            .position(|op| op.event == Some(event))
        else {
            self.consume_event(event);
            return Ok(());
        };
        if matches!(self.operations[index].kind, OperationKind::Save(_)) {
            return Err(RuntimeError::Denied);
        }
        if !matches!(self.operations[index].state, OperationState::Ready(_)) {
            return Err(RuntimeError::Busy);
        }
        let token = match self.operations[index].kind {
            OperationKind::Resolve { token, .. } | OperationKind::Prepare(token) => Some(token),
            _ => None,
        };
        if let Some(token) = token {
            let window = self.operations[index].window;
            let slot = self.slot(window)?;
            if self.sessions[slot].app.projection().continuation == Some(token) {
                let context = self.current_context(window, token.request().0)?;
                self.sessions[slot]
                    .app
                    .dispatch(
                        &mut self.coordinator,
                        context,
                        ApplicationCommand::ResolveEffect(EffectReply {
                            token,
                            decision: EffectDecision::Cancel,
                        }),
                    )
                    .map_err(|error| RuntimeError::Application(error.failure))?;
            }
        }
        let op = self.operations.remove(index).expect("validated operation");
        if self.barrier == Some(op.id) {
            self.barrier = None;
        }
        if let OperationState::Ready(value) = op.state {
            match *value {
                WorkerValue::Prepared(candidate) => self.retire(Retired::Candidate(*candidate))?,
                WorkerValue::Destination(destination) => {
                    if let Some(lease) =
                        self.leases.iter_mut().find(|lease| lease.grant == op.grant)
                    {
                        lease.destination = Some(destination);
                    }
                }
                _ => {}
            }
        }
        self.consume_event(event);
        self.release_grant(op.grant)?;
        self.pump()?;
        Ok(())
    }
}
