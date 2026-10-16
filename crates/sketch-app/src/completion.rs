use crate::{operations::Phase, work::Payload, *};
impl PortableApp {
    /// Rejected completion retains the caller's unique result. Successful completion
    /// stores a ready result only; live application state changes in apply_ready.
    pub fn complete_work(&mut self, result: &mut Option<WorkResult>) -> Result<(), AppError> {
        let info = result.as_ref().map(WorkResult::info);
        let completed = self.complete_work_inner(result);
        if let Some(info) = info {
            let outcome = match &completed {
                Err(error) => DiagnosticOutcome::Rejected(*error),
                Ok(()) => match self
                    .operation(info.operation)
                    .ok()
                    .map(|i| &self.operations[i].phase)
                {
                    Some(Phase::Ready(Payload::Save(Some(value)))) => {
                        DiagnosticOutcome::Write(value.result())
                    }
                    Some(Phase::Ready(Payload::Failure(Some(value)))) => {
                        DiagnosticOutcome::ReplacementFailed(value.failure())
                    }
                    _ => DiagnosticOutcome::WorkCompleted,
                },
            };
            self.diagnostics.record(self.work_diagnostic(info, outcome));
        }
        completed
    }
    fn complete_work_inner(&mut self, result: &mut Option<WorkResult>) -> Result<(), AppError> {
        let held = result.as_ref().ok_or(AppError::Invalid)?;
        if held.stamp.app != self.identity {
            return Err(AppError::Stale);
        }
        let index = self.operation(held.stamp.info.operation)?;
        let Phase::Active(kind) = self.operations[index].phase else {
            return Err(AppError::Stale);
        };
        if self.stamp(index, kind)?.info != held.stamp.info {
            return Err(AppError::Stale);
        }
        let valid = matches!(
            (kind, &held.payload),
            (WorkKind::Write, Payload::Write(_))
                | (
                    WorkKind::Read | WorkKind::PrepareNew,
                    Payload::Candidate(_) | Payload::Failure(_)
                )
                | (WorkKind::Retire, Payload::Retired)
        );
        if !valid {
            return Err(AppError::Invalid);
        }
        let retirement_id = if kind == WorkKind::Retire {
            Some(OperationId(
                crate::ids::reserve(&mut self.next_operation)?,
                self.identity,
            ))
        } else {
            None
        };
        let payload = match result.take().unwrap().payload {
            Payload::Write(receipt) => {
                if let Err(rejected) = self.coordinator.finish(receipt) {
                    *result = Some(WorkResult {
                        stamp: Box::new(self.stamp(index, kind)?),
                        payload: Payload::Write(rejected.into_receipt()),
                    });
                    return Err(AppError::Invalid);
                }
                let token = self.operations[index].save.expect("write token");
                Payload::Save(Some(self.coordinator.take_completion(token)?))
            }
            Payload::Retired => {
                let handle = self.operations.swap_remove(index).session;
                self.retire_detached(handle, retirement_id.unwrap());
                self.release_unused();
                return Ok(());
            }
            other => other,
        };
        self.operations[index].phase = Phase::Ready(payload);
        Ok(())
    }
}
