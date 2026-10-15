use super::{NativeRuntime, owner_types::*, work::*};
impl NativeRuntime {
    /// Poisoned owners never receive further model mutations. Accepted I/O still settles.
    pub(super) fn drain_detached(&mut self) -> Result<(), RuntimeError> {
        for _ in 0..32 {
            let next = self.operations.iter().position(|op| {
                matches!(op.state, OperationState::Ready(_))
                    && self
                        .sessions
                        .iter()
                        .any(|slot| slot.window == op.window && slot.detached)
            });
            let Some(index) = next else {
                break;
            };
            let op = self.operations.remove(index).expect("checked operation");
            if self.barrier == Some(op.id) {
                self.barrier = None;
            }
            if let Some(event) = op.event {
                self.consume_event(event);
            }
            if let OperationState::Ready(value) = op.state {
                match *value {
                    WorkerValue::Prepared(candidate) => {
                        self.retire(Retired::Candidate(*candidate))?
                    }
                    WorkerValue::Destination(destination) => {
                        if let Some(lease) =
                            self.leases.iter_mut().find(|lease| lease.grant == op.grant)
                        {
                            lease.destination = Some(destination);
                        }
                    }
                    // Trusted completion is terminal; poisoned app is retired without claiming clean.
                    WorkerValue::Completed(_, _) => {}
                    _ => {}
                }
            }
            self.release_grant(op.grant)?;
        }
        let mut index = 0;
        while index < self.sessions.len() {
            let window = self.sessions[index].window;
            if self.sessions[index].detached
                && !self.operations.iter().any(|op| op.window == window)
            {
                let slot = self.sessions.remove(index);
                self.retire(Retired::Session(slot.app))?;
            } else {
                index += 1;
            }
        }
        Ok(())
    }
}
