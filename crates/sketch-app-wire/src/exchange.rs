use crate::{work::WorkState, *};
impl WireApp {
    /// A regular call cannot issue work: both bytes and its unique sidecar are returned here.
    pub fn take_work(&mut self, bytes: &[u8]) -> Result<(Reply, Option<WireWork>), WireError> {
        let packet = decode(bytes)?;
        if !matches!(packet.command, Request::TakeWork) {
            return Err(WireError::transport(3));
        }
        let mut reply = Reply::reserve(&self.budget, packet.opcode, packet.request, 4096)?;
        let fresh = crate::identity::reserve(2)?;
        let work = self.app.take_work()?;
        reply.u32(u32::from(work.is_some()));
        reply.u32(0);
        let work = work.map(|work| {
            let info = work.info();
            let operation = self.register_operation(
                info.operation,
                info.session,
                fresh,
                info.kind == WorkKind::Write,
                true,
            );
            let work = WireWork {
                app: self.id,
                operation,
                session: self.session_ref(info.session),
                state: Some(WorkState::Issued(work)),
                budget: self.budget.clone(),
            };
            work.info_reply(&mut reply).expect("newly issued work");
            work
        });
        if work.is_none() {
            self.prune_sessions();
        }
        Ok((reply.finish(), work))
    }
    /// Rejection leaves the same caller-owned result available for its actual owner.
    pub fn complete(
        &mut self,
        bytes: &[u8],
        result: &mut Option<WireResult>,
    ) -> Result<Reply, WireError> {
        let packet = decode(bytes)?;
        if !matches!(packet.command, Request::Complete) {
            return Err(WireError::transport(3));
        }
        let reply = Reply::reserve(&self.budget, packet.opcode, packet.request, 4096)?;
        let held = result.as_mut().ok_or(WireError::malformed())?;
        if held.app != self.id {
            return Err(WireError::plain(11));
        }
        let info = held.inner.as_ref().ok_or(WireError::plain(2))?.info();
        if self.operation(held.operation)? != info.operation {
            return Err(WireError::plain(7));
        }
        self.app.complete_work(&mut held.inner)?;
        if info.kind == WorkKind::Retire {
            self.operations.retain(|op| op.handle != info.operation);
        } else if let Some(op) = self
            .operations
            .iter_mut()
            .find(|op| op.handle == info.operation)
        {
            op.active = false;
        }
        *result = None;
        Ok(reply.finish())
    }
    fn prune_sessions(&mut self) {
        self.sessions.retain(|s| {
            self.app.projection(s.handle).is_ok()
                || self.operations.iter().any(|op| op.session == s.handle)
        });
    }
    pub fn shutdown_ready(&self) -> bool {
        self.app.stats().shutdown_ready()
            && self.budget.stats().0 == 0
            && self.scenes.leases.is_empty()
    }
}
