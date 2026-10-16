use crate::*;
impl WireApp {
    pub(crate) fn diagnostic(&mut self, reply: &mut Reply) {
        let event = self.app.take_diagnostic();
        reply.u32(u32::from(event.is_some()));
        reply.u32(0);
        let Some(event) = event else {
            return;
        };
        reply.diagnostic_record(event, event.session.map_or(0, |s| self.session_ref(s)));
    }
    pub(crate) fn stats_reply(&self, reply: &mut Reply) {
        let s = self.app.stats();
        let b = self.budget.stats();
        for v in [
            s.sessions as u64,
            s.operations as u64,
            s.resource_leases as u64,
            s.buffers as u64,
            s.buffer_bytes as u64,
            s.outstanding_saves as u64,
            s.diagnostic_drops,
            b.0 as u64,
            b.1 as u64,
            self.scenes.leases.len() as u64,
            self.scenes.bytes as u64,
            self.scenes.geometries.len() as u64,
        ] {
            reply.u64(v);
        }
    }
}

impl Reply {
    pub(crate) fn diagnostic_record(&mut self, event: AppDiagnostic, session: u64) {
        let reply = self;
        reply.u64(event.app);
        reply.u64(event.operation.map_or(0, OperationId::get));
        reply.u64(session);
        reply.u64(event.request.0);
        reply.u32(event.provider.map_or(0, |p| p.0));
        let mut result = 0;
        let mut durability = 0;
        let mut write_failure = 0;
        let mut failure = Failure::default();
        let kind = match event.outcome {
            DiagnosticOutcome::WorkCompleted => 1,
            DiagnosticOutcome::Write(value) => {
                (result, durability, write_failure) = write_tags::save_result(value);
                2
            }
            DiagnosticOutcome::ReplacementFailed(value) => {
                failure = value.into();
                3
            }
            DiagnosticOutcome::Cancelled => 4,
            DiagnosticOutcome::Rejected(error) => {
                let error = WireError::from(error);
                failure = error.failure;
                result = error.status;
                5
            }
        };
        for v in [kind, result, durability, write_failure, 0] {
            reply.u32(v);
        }
        reply.raw(&failure.bytes());
    }
}
