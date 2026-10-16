use crate::{
    operations::Phase,
    work::{Payload, Retired},
    *,
};
impl PortableApp {
    pub fn cancel_operation(&mut self, id: OperationId) -> Result<(), AppError> {
        let diagnostic = self.operation_diagnostic(id, DiagnosticOutcome::Cancelled);
        let result = self.cancel_operation_inner(id);
        if let Some(mut diagnostic) = diagnostic {
            if let Err(error) = result {
                diagnostic.outcome = DiagnosticOutcome::Rejected(error);
            }
            self.diagnostics.record(diagnostic);
        }
        result
    }
    fn cancel_operation_inner(&mut self, id: OperationId) -> Result<(), AppError> {
        let index = self.operation(id)?;
        if matches!(
            self.operations[index].phase,
            Phase::Retire(_) | Phase::Active(WorkKind::Retire)
        ) {
            return Err(AppError::Busy);
        }
        if let Some(token) = self.operations[index].save {
            self.coordinator.cancel(token)?;
            if matches!(self.operations[index].phase, Phase::Write(_)) {
                self.operations[index].phase = Phase::Ready(Payload::Save(Some(
                    self.coordinator.take_completion(token)?,
                )));
            }
            return Ok(());
        }
        let free_id = if matches!(self.operations[index].phase, Phase::Active(_)) {
            None
        } else {
            Some(OperationId(
                crate::ids::reserve(&mut self.next_operation)?,
                self.identity,
            ))
        };
        let handle = self.operations[index].session;
        self.cancel_continuation(index)?;
        if matches!(self.operations[index].phase, Phase::Active(_)) {
            return Ok(());
        }
        let free_id = free_id.expect("nonactive retirement reserved");
        let op = self.operations.swap_remove(index);
        if let Phase::Ready(Payload::Candidate(candidate)) = op.phase {
            self.operations.push(crate::operations::Operation {
                phase: Phase::Retire(RetiredState {
                    stamp: None,
                    states: vec![Retired::Candidate(candidate)],
                }),
                resource: None,
                ..op
            });
        }
        self.retire_detached(handle, free_id);
        self.release_unused();
        Ok(())
    }
    pub(crate) fn cancel_continuation(&mut self, index: usize) -> Result<(), AppError> {
        let handle = self.operations[index].session;
        let slot = self.slot(handle)?;
        if let Some(token) = self.operations[index].continuation
            && self.sessions[slot].app.projection().continuation == Some(token)
        {
            let context = self.context(slot, self.operations[index].request);
            self.sessions[slot].app.dispatch(
                &mut self.coordinator,
                context,
                ApplicationCommand::ResolveEffect(EffectReply {
                    token,
                    decision: EffectDecision::Cancel,
                }),
            )?;
        }
        Ok(())
    }
}
