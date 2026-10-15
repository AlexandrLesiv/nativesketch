use super::{NativeRuntime, owner_types::*, work::*};
use sketch_core::application::{routing::*, *};
impl NativeRuntime {
    pub fn apply_event(&mut self, window: u64, event: u64) -> Result<ActionResult, RuntimeError> {
        self.check_owner()?;
        let op_index = self.event_operation(event)?;
        if self.operations[op_index].window != window || self.operations[op_index].cancelled {
            return Err(RuntimeError::Stale);
        }
        let slot = self.slot(window)?;
        if self.sessions[slot].detached {
            return Err(RuntimeError::Stale);
        }
        let current = self.sessions[slot].app.projection().editor;
        if matches!(
            self.operations[op_index].kind,
            OperationKind::Resolve { .. } | OperationKind::Prepare(_)
        ) && current.interaction != InteractionState::Idle
        {
            return Err(RuntimeError::Busy);
        }
        if self.operations.len() + 2 > self.config.operations + self.config.sessions {
            return Err(RuntimeError::Capacity);
        }
        let context = self.current_context(window, self.operations[op_index].request)?;
        let op = &mut self.operations[op_index];
        let OperationState::Ready(value) = &mut op.state else {
            return Err(RuntimeError::Busy);
        };
        let command = match (value.as_mut(), &op.kind) {
            (WorkerValue::Destination(destination), OperationKind::Resolve { token, source }) => {
                let lease = self
                    .leases
                    .iter_mut()
                    .find(|lease| lease.grant == op.grant)
                    .ok_or(RuntimeError::Invalid)?;
                lease.destination = Some(*destination);
                ApplicationCommand::ResolveEffect(EffectReply {
                    token: *token,
                    decision: if *source {
                        EffectDecision::Source(*destination)
                    } else {
                        EffectDecision::Destination(*destination)
                    },
                })
            }
            (WorkerValue::Prepared(candidate), OperationKind::Prepare(_)) => {
                ApplicationCommand::Install(candidate)
            }
            (WorkerValue::ReplacementFailed(failure), OperationKind::Prepare(_)) => {
                ApplicationCommand::ReplacementFailed(failure)
            }
            (WorkerValue::Completed(completion, _), OperationKind::Save(_)) => {
                ApplicationCommand::CompleteSave(completion)
            }
            (WorkerValue::Failure(_), _) => {
                let token = match op.kind {
                    OperationKind::Resolve { token, .. } | OperationKind::Prepare(token) => token,
                    _ => return Err(RuntimeError::Invalid),
                };
                if self.sessions[slot].app.projection().continuation != Some(token) {
                    return Err(RuntimeError::Stale);
                }
                ApplicationCommand::ResolveEffect(EffectReply {
                    token,
                    decision: EffectDecision::Cancel,
                })
            }
            _ => return Err(RuntimeError::Invalid),
        };
        let mut outcome = self.sessions[slot]
            .app
            .dispatch(&mut self.coordinator, context, command)
            .map_err(|error| RuntimeError::Application(error.failure))?;
        if matches!(value.as_ref(), WorkerValue::Failure(_)) {
            outcome.effect = Some(ApplicationEffect::Failure(ApplicationFailure::Write(
                sketch_core::application::storage::StorageFailure::Access,
            )));
        }
        let op = self
            .operations
            .remove(op_index)
            .expect("validated operation");
        if self.barrier == Some(op.id) {
            self.barrier = None;
        }
        self.consume_event(event);
        let destination = outcome.projection.save.destination();
        if outcome.changes.session_replaced
            || outcome
                .save
                .as_ref()
                .is_some_and(|save| destination == Some(save.token.destination()))
        {
            self.bind_grant(window, op.grant, destination)?;
        }
        if outcome.changes.session_replaced {
            self.sessions[slot].recovered_from = op.recovery_source.clone();
            self.sessions[slot].requested_recovery = None;
        }
        if let Some(save) = &outcome.save {
            self.checkpoint_complete(window, op.checkpoint.as_ref(), save)?;
            self.saved_recovery(window, save)?;
        }
        if let Some(save) = &outcome.save
            && matches!(
                save.token.purpose(),
                storage::SavePurpose::Save
                    | storage::SavePurpose::SaveAs
                    | storage::SavePurpose::ExportCopy
            )
            && let storage::SaveResult::Failed(failure) | storage::SaveResult::Uncertain(failure) =
                save.result
        {
            outcome.effect = Some(ApplicationEffect::Failure(ApplicationFailure::Write(
                failure,
            )));
        }
        let mut result = self.action(window, outcome, op.grant, op.retry)?;
        if result.operation == 0 {
            result.operation = op.id;
        }
        result.request = op.request;
        result.operation_kind = op.boundary;
        self.release_grant(op.grant)?;
        self.pump()?;
        Ok(result)
    }
    pub(super) fn event_operation(&self, event: u64) -> Result<usize, RuntimeError> {
        if !self.delivered.iter().any(|item| item.id == event) {
            return Err(RuntimeError::Stale);
        }
        self.operations
            .iter()
            .position(|op| op.event == Some(event))
            .ok_or(RuntimeError::Stale)
    }
    pub(super) fn consume_event(&mut self, event: u64) {
        self.delivered.retain(|item| item.id != event);
        self.events.retain(|item| item.id != event);
    }
}
