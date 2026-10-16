use crate::*;
use std::collections::VecDeque;
/// Typed redaction boundary: there are no paths, strings, samples or document bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AppDiagnostic {
    pub app: u64,
    pub operation: Option<OperationId>,
    pub session: Option<SessionHandle>,
    pub request: RequestId,
    pub provider: Option<ProviderId>,
    pub outcome: DiagnosticOutcome,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticOutcome {
    WorkCompleted,
    Write(SaveResult),
    ReplacementFailed(ApplicationFailure),
    Cancelled,
    Rejected(AppError),
}
pub(crate) struct Diagnostics {
    events: VecDeque<AppDiagnostic>,
    capacity: usize,
    pub dropped: u64,
}
impl Diagnostics {
    pub fn new(capacity: usize) -> Self {
        Self {
            events: VecDeque::with_capacity(capacity),
            capacity,
            dropped: 0,
        }
    }
    pub fn record(&mut self, event: AppDiagnostic) {
        if self.events.len() == self.capacity {
            self.events.pop_front();
            self.dropped = self.dropped.saturating_add(1);
        }
        self.events.push_back(event);
    }
}
impl PortableApp {
    /// Drain on host logging boundary, never from a per-sample callback.
    pub fn take_diagnostic(&mut self) -> Option<AppDiagnostic> {
        self.diagnostics.events.pop_front()
    }
    pub(crate) fn operation_diagnostic(
        &self,
        id: OperationId,
        outcome: DiagnosticOutcome,
    ) -> Option<AppDiagnostic> {
        let op = &self.operations[self.operation(id).ok()?];
        Some(AppDiagnostic {
            app: self.identity,
            operation: Some(id),
            session: Some(op.session),
            request: op.request,
            provider: op
                .resource
                .and_then(|id| self.resource(id).ok().map(|r| r.descriptor.provider)),
            outcome,
        })
    }
    pub(crate) fn work_diagnostic(
        &self,
        info: WorkInfo,
        outcome: DiagnosticOutcome,
    ) -> AppDiagnostic {
        AppDiagnostic {
            app: self.identity,
            operation: Some(info.operation),
            session: Some(info.session),
            request: info.request,
            provider: info.resource.map(|r| r.provider),
            outcome,
        }
    }
    pub(crate) fn resource_rejected(
        &mut self,
        handle: SessionHandle,
        context: CommandContext,
        descriptor: ResourceDescriptor,
        result: &Result<AppOutcome, AppError>,
    ) {
        if let Err(error) = result {
            self.diagnostics.record(AppDiagnostic {
                app: self.identity,
                operation: None,
                session: Some(handle),
                request: context.request,
                provider: Some(descriptor.provider),
                outcome: DiagnosticOutcome::Rejected(*error),
            });
        }
    }
}
