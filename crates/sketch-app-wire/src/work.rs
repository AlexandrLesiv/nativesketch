use crate::{
    budget::{Budget, Permit},
    *,
};
use std::sync::Arc;
#[derive(Debug)]
pub(crate) enum WorkState {
    Issued(WorkItem),
    Encoded(EncodedWrite),
}
/// Unique issued ownership, movable to one worker; no Clone and no caller-made constructor.
#[derive(Debug)]
pub struct WireWork {
    pub(crate) app: u64,
    pub(crate) operation: u64,
    pub(crate) session: u64,
    pub(crate) state: Option<WorkState>,
    pub(crate) budget: Arc<Budget>,
}
#[derive(Debug)]
pub struct WireResult {
    pub(crate) app: u64,
    pub(crate) operation: u64,
    pub(crate) inner: Option<WorkResult>,
    pub(crate) _identity: u64,
}
#[derive(Debug)]
pub struct InputBytes {
    pub(crate) bytes: Vec<u8>,
    pub(crate) _permit: Permit,
    pub(crate) app: u64,
}
#[derive(Debug)]
pub struct WorkerReply {
    pub reply: Reply,
    pub result: Option<WireResult>,
}
impl WireWork {
    /// Copy only on the worker, within its read allowance and the shared transport budget.
    pub fn copy_input(&self, bytes: &[u8]) -> Result<InputBytes, WireError> {
        let Some(WorkState::Issued(WorkItem::Read(read))) = &self.state else {
            return Err(WireError::transport(3));
        };
        if bytes.len() > read.maximum_bytes().saturating_add(1) {
            return Err(WireError::capacity());
        }
        let permit = self.budget.reserve(bytes.len())?;
        let mut copy = Vec::new();
        copy.try_reserve_exact(bytes.len())
            .map_err(|_| WireError::capacity())?;
        copy.extend_from_slice(bytes);
        Ok(InputBytes {
            bytes: copy,
            _permit: permit,
            app: self.app,
        })
    }
    pub(crate) fn info(&self) -> Result<WorkInfo, WireError> {
        match &self.state {
            Some(WorkState::Issued(work)) => Ok(work.info()),
            Some(WorkState::Encoded(work)) => Ok(work.info()),
            None => Err(WireError::plain(2)),
        }
    }
    pub(crate) fn info_reply(&self, reply: &mut Reply) -> Result<(), WireError> {
        let info = self.info()?;
        reply.u64(self.operation);
        reply.u64(self.session);
        reply.u64(info.request.0);
        reply.u32(write_tags::work(info.kind));
        reply.u32(u32::from(info.resource.is_some()));
        reply.resource(info.resource);
        let maximum = match &self.state {
            Some(WorkState::Issued(WorkItem::Read(work))) => work.maximum_bytes(),
            _ => 0,
        };
        reply.u64(maximum as u64);
        Ok(())
    }
}
