use crate::*;
pub(crate) enum Phase {
    Write(SaveToken),
    Read(ReplacementTicket),
    New(ReplacementTicket),
    Active(WorkKind),
    Ready(crate::work::Payload),
    Retire(RetiredState),
}
pub(crate) struct Operation {
    pub id: OperationId,
    pub session: SessionHandle,
    pub request: RequestId,
    pub resource: Option<DestinationHandle>,
    pub phase: Phase,
    pub save: Option<SaveToken>,
    pub continuation: Option<EffectToken>,
}
impl PortableApp {
    pub(crate) fn reserve_operation(&mut self) -> Result<OperationId, AppError> {
        if self.operations.len() >= self.limits.operations {
            return Err(AppError::Capacity);
        }
        Ok(OperationId(
            crate::ids::reserve(&mut self.next_operation)?,
            self.identity,
        ))
    }
    pub(crate) fn operation(&self, id: OperationId) -> Result<usize, AppError> {
        self.operations
            .iter()
            .position(|op| op.id == id)
            .ok_or(AppError::Stale)
    }
}
