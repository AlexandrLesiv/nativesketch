use crate::{
    operations::Phase,
    work::{Payload, Retired},
    *,
};
impl PortableApp {
    pub fn apply_ready(
        &mut self,
        handle: SessionHandle,
        id: OperationId,
    ) -> Result<AppOutcome, AppError> {
        let diagnostic = self.operation_diagnostic(id, DiagnosticOutcome::WorkCompleted);
        let result = self.apply_ready_inner(handle, id);
        if let (Some(mut diagnostic), Err(error)) = (diagnostic, &result) {
            diagnostic.outcome = DiagnosticOutcome::Rejected(*error);
            self.diagnostics.record(diagnostic);
        }
        result
    }

    fn apply_ready_inner(
        &mut self,
        handle: SessionHandle,
        id: OperationId,
    ) -> Result<AppOutcome, AppError> {
        let op = self.operation(id)?;
        if self.operations[op].session != handle {
            return Err(AppError::Stale);
        }
        if !matches!(self.operations[op].phase, Phase::Ready(_)) {
            return Err(AppError::Busy);
        }
        let slot = self.slot(handle)?;
        let next = OperationId(
            crate::ids::reserve(&mut self.next_operation)?,
            self.identity,
        );
        let request = self.operations[op].request;
        let context = self.context(slot, request);
        let Phase::Ready(mut payload) = std::mem::replace(
            &mut self.operations[op].phase,
            Phase::Active(WorkKind::Retire),
        ) else {
            unreachable!()
        };
        let command = match &mut payload {
            Payload::Save(completion) => ApplicationCommand::CompleteSave(completion),
            Payload::Candidate(candidate) => ApplicationCommand::Install(candidate),
            Payload::Failure(failure) => ApplicationCommand::ReplacementFailed(failure),
            _ => unreachable!("ready application payload"),
        };
        let result = self.sessions[slot]
            .app
            .dispatch(&mut self.coordinator, context, command);
        let outcome = match result {
            Ok(outcome) => outcome,
            Err(error) => {
                if matches!(
                    error.failure,
                    ApplicationFailure::InvalidEffect | ApplicationFailure::StaleDocument
                ) {
                    if let Err(cleanup) = self.cancel_continuation(op) {
                        self.operations[op].phase = Phase::Ready(payload);
                        return Err(cleanup);
                    }
                    self.reject_stale(op, payload);
                    self.retire_detached(handle, next);
                    self.release_unused();
                } else {
                    self.operations[op].phase = Phase::Ready(payload);
                }
                return Err(error.into());
            }
        };
        let resource = self.operations.swap_remove(op).resource;
        let outcome = self.capture(handle, request, outcome, resource, Some(next))?;
        self.retire_detached(handle, next);
        self.release_unused();
        Ok(outcome)
    }
    fn reject_stale(&mut self, index: usize, payload: Payload) {
        if let Payload::Candidate(candidate) = payload {
            let op = &mut self.operations[index];
            op.resource = None;
            op.phase = Phase::Retire(RetiredState {
                stamp: None,
                states: vec![Retired::Candidate(candidate)],
            });
        } else {
            self.operations.swap_remove(index);
        }
    }
}
