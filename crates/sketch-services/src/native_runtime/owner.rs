use super::{NativeIo, RuntimeConfig, owner_types::*};
use sketch_core::application::{routing::*, storage::*, *};
use std::{
    collections::VecDeque,
    thread::{self, ThreadId},
    time::Instant,
};
pub struct NativeRuntime {
    pub(super) owner: ThreadId,
    pub(super) config: RuntimeConfig,
    pub(super) io: NativeIo,
    pub(super) coordinator: SaveCoordinator,
    pub(super) sessions: Vec<SessionSlot>,
    pub(super) operations: VecDeque<PendingOperation>,
    pub(super) events: VecDeque<NativeEvent>,
    pub(super) delivered: Vec<NativeEvent>,
    pub(super) leases: Vec<Lease>,
    pub(super) barrier: Option<u64>,
    pub(super) next_id: u64,
    pub(super) started: Instant,
    pub(super) stopping: bool,
    pub(super) shutdown_notified: bool,
    pub(super) worker_active: bool,
    pub(super) recovery_available: bool,
    pub(super) drops: u64,
    pub(super) sink_failures: u64,
}
impl NativeRuntime {
    pub fn new(config: RuntimeConfig) -> Result<Self, RuntimeError> {
        if !config.validate() {
            return Err(RuntimeError::Invalid);
        }
        let coordinator = SaveCoordinator::new(StorageLimits {
            jobs: config.operations,
            snapshot_bytes: config.snapshot_bytes,
        })
        .map_err(|_| RuntimeError::Invalid)?;
        let io = NativeIo::spawn(config.clone()).map_err(|_| RuntimeError::Stopped)?;
        Ok(Self {
            owner: thread::current().id(),
            config,
            io,
            coordinator,
            sessions: Vec::new(),
            operations: VecDeque::new(),
            events: VecDeque::new(),
            delivered: Vec::new(),
            leases: Vec::new(),
            barrier: None,
            next_id: 1,
            started: Instant::now(),
            stopping: false,
            shutdown_notified: false,
            worker_active: false,
            recovery_available: false,
            drops: 0,
            sink_failures: 0,
        })
    }
    pub fn check_owner(&self) -> Result<(), RuntimeError> {
        if self.owner == thread::current().id() {
            Ok(())
        } else {
            Err(RuntimeError::Owner)
        }
    }
    pub(super) fn id(&mut self) -> Result<u64, RuntimeError> {
        let id = self.next_id;
        self.next_id = id.checked_add(1).ok_or(RuntimeError::Capacity)?;
        Ok(id)
    }
    pub fn create(
        &mut self,
        capabilities: Capabilities,
        tool: ToolId,
        brush: BrushSettings,
    ) -> Result<u64, RuntimeError> {
        self.check_owner()?;
        if self.stopping {
            return Err(RuntimeError::Stopped);
        }
        if self.sessions.len() >= self.config.sessions {
            return Err(RuntimeError::Capacity);
        }
        let app = ApplicationSession::new(capabilities, tool, brush, None)
            .map_err(|error| RuntimeError::Application(ApplicationFailure::Editor(error)))?;
        let window = self.id()?;
        self.sessions.push(SessionSlot {
            window,
            app,
            bound_grant: 0,
            close_confirmed: false,
            pristine: true,
            detached: false,
            recovery: None,
            recovered_from: None,
            requested_recovery: None,
            last_checkpoint: None,
            schedule: Default::default(),
        });
        Ok(window)
    }
    pub fn preparing_operations(&self, window: u64) -> Result<u32, RuntimeError> {
        self.session(window)?;
        Ok(self
            .operations
            .iter()
            .filter(|op| {
                op.window == window
                    && matches!(
                        op.kind,
                        OperationKind::Resolve { .. } | OperationKind::Prepare(_)
                    )
            })
            .count() as u32)
    }
    pub fn session(&self, window: u64) -> Result<&SessionSlot, RuntimeError> {
        self.check_owner()?;
        self.sessions
            .iter()
            .find(|slot| slot.window == window && !slot.detached)
            .ok_or(RuntimeError::Stale)
    }
    pub(super) fn slot(&self, window: u64) -> Result<usize, RuntimeError> {
        self.sessions
            .iter()
            .position(|slot| slot.window == window)
            .ok_or(RuntimeError::Stale)
    }
    pub(super) fn capacity(&self) -> Result<(), RuntimeError> {
        if self.operations.len() >= self.config.operations
            || self.events.len()
                + self.delivered.len()
                + self.operations.len() * 2
                + self.sessions.len()
                >= self.config.events
        {
            Err(RuntimeError::Capacity)
        } else {
            Ok(())
        }
    }
    pub fn context(
        &self,
        window: u64,
        session: u64,
        request: u64,
        interaction: u64,
    ) -> Result<CommandContext, RuntimeError> {
        let state = self.session(window)?.app.projection().editor;
        if state.session.get() != session {
            return Err(RuntimeError::Stale);
        }
        let active = match state.interaction {
            InteractionState::Idle => None,
            InteractionState::Drawing(id) | InteractionState::Transforming(id) => Some(id),
        };
        if active.map_or(0, |id| id.get()) != interaction {
            return Err(RuntimeError::Stale);
        }
        Ok(CommandContext {
            session: state.session,
            request: RequestId(request),
            interaction: active,
        })
    }
    pub(super) fn current_context(
        &self,
        window: u64,
        request: u64,
    ) -> Result<CommandContext, RuntimeError> {
        let state = self.sessions[self.slot(window)?].app.projection().editor;
        let interaction = match state.interaction {
            InteractionState::Idle => None,
            InteractionState::Drawing(id) | InteractionState::Transforming(id) => Some(id),
        };
        Ok(CommandContext {
            session: state.session,
            request: RequestId(request),
            interaction,
        })
    }
}
