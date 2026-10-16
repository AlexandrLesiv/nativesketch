use crate::{buffer::Budget, operations::Operation, resources::Resource, *};
use std::{
    marker::PhantomData,
    rc::Rc,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};
static NEXT_APP: AtomicU64 = AtomicU64::new(1);
pub(crate) struct Slot {
    pub handle: SessionHandle,
    pub app: ApplicationSession,
    pub bound_resource: Option<DestinationHandle>,
    pub choice: Option<crate::dispatch::Choice>,
    pub pristine: bool,
    pub detached: bool,
    pub close_confirmed: bool,
}
/// Not Send/Sync: one executor owns mutations. Workers contain no owner references.
pub struct PortableApp {
    pub(crate) identity: u64,
    pub(crate) limits: AppLimits,
    pub(crate) sessions: Vec<Slot>,
    pub(crate) operations: Vec<Operation>,
    pub(crate) resources: Vec<Resource>,
    pub(crate) coordinator: SaveCoordinator,
    pub(crate) budget: Arc<Budget>,
    pub(crate) diagnostics: crate::diagnostics::Diagnostics,
    pub(crate) next_session: u64,
    pub(crate) next_operation: u64,
    pub(crate) next_resource: u64,
    pub(crate) next_destination: u64,
    owner: PhantomData<Rc<()>>,
}
impl PortableApp {
    pub fn new(limits: AppLimits) -> Result<Self, AppError> {
        let limits = limits.validate()?;
        let identity = NEXT_APP
            .try_update(Ordering::Relaxed, Ordering::Relaxed, |n| n.checked_add(1))
            .map_err(|_| AppError::Capacity)?;
        Ok(Self {
            identity,
            limits,
            sessions: Vec::new(),
            operations: Vec::new(),
            resources: Vec::new(),
            coordinator: SaveCoordinator::new(limits.storage)?,
            budget: Budget::new(limits),
            diagnostics: crate::diagnostics::Diagnostics::new(limits.operations),
            next_session: 1,
            next_operation: 1,
            next_resource: 1,
            next_destination: 1,
            owner: PhantomData,
        })
    }
    pub(crate) fn slot(&self, handle: SessionHandle) -> Result<usize, AppError> {
        self.sessions
            .iter()
            .position(|s| s.handle == handle)
            .ok_or(AppError::Stale)
    }
    pub(crate) fn attached(&self, handle: SessionHandle) -> Result<usize, AppError> {
        let index = self.slot(handle)?;
        if self.sessions[index].detached || self.sessions[index].close_confirmed {
            Err(AppError::Stale)
        } else {
            Ok(index)
        }
    }
    pub(crate) fn context(&self, index: usize, request: RequestId) -> CommandContext {
        let state = self.sessions[index].app.projection().editor;
        CommandContext {
            session: state.session,
            request,
            interaction: match state.interaction {
                InteractionState::Idle => None,
                InteractionState::Drawing(id) | InteractionState::Transforming(id) => Some(id),
            },
        }
    }
    pub fn stats(&self) -> AppStats {
        AppStats {
            sessions: self.sessions.len(),
            operations: self.operations.len(),
            resource_leases: self.resources.len(),
            buffers: self.budget.count.load(Ordering::Acquire),
            buffer_bytes: self.budget.bytes.load(Ordering::Acquire),
            outstanding_saves: self.coordinator.outstanding(),
            diagnostic_drops: self.diagnostics.dropped,
        }
    }
}
