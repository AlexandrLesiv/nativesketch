use crate::{budget::Budget, *};
use std::sync::Arc;
pub(crate) struct Session {
    pub id: u64,
    pub handle: SessionHandle,
    pub effect: Option<(u64, EffectToken)>,
}
pub(crate) struct Op {
    pub id: u64,
    pub handle: OperationId,
    pub session: SessionHandle,
    pub active: bool,
    pub write: bool,
}
/// One mutation owner; inherits !Send/!Sync from PortableApp. No host policy or threads.
pub struct WireApp {
    pub(crate) id: u64,
    pub(crate) app: PortableApp,
    pub(crate) sessions: Vec<Session>,
    pub(crate) operations: Vec<Op>,
    pub(crate) budget: Arc<Budget>,
    pub(crate) limits: AppLimits,
}
impl WireApp {
    pub fn bootstrap(bytes: &[u8], limits: TransportLimits) -> Result<Self, WireError> {
        let packet = decode(bytes)?;
        let Request::Bootstrap(config) = packet.command else {
            return Err(WireError::transport(3));
        };
        let limits = limits.validate()?;
        // Tight transport bounds supplement the facade and prevent hostile oversized bootstrap.
        if config.buffer_bytes > 512 * 1024 * 1024
            || config.storage.snapshot_bytes > 512 * 1024 * 1024
        {
            return Err(WireError::capacity());
        }
        let app = PortableApp::new(config)?;
        let mut sessions = Vec::new();
        sessions
            .try_reserve_exact(config.sessions)
            .map_err(|_| WireError::capacity())?;
        let mut operations = Vec::new();
        operations
            .try_reserve_exact(config.operations)
            .map_err(|_| WireError::capacity())?;
        Ok(Self {
            id: crate::identity::reserve(1)?,
            app,
            sessions,
            operations,
            budget: Budget::new(limits.buffers, limits.buffer_bytes),
            limits: config,
        })
    }
    pub(crate) fn session(&self, id: u64) -> Result<SessionHandle, WireError> {
        self.sessions
            .iter()
            .find(|s| s.id == id)
            .map(|s| s.handle)
            .ok_or(WireError::plain(7))
    }
    pub(crate) fn session_ref(&self, handle: SessionHandle) -> u64 {
        self.sessions
            .iter()
            .find(|s| s.handle == handle)
            .map_or(0, |s| s.id)
    }
    pub(crate) fn operation(&self, id: u64) -> Result<OperationId, WireError> {
        self.operations
            .iter()
            .find(|o| o.id == id)
            .map(|o| o.handle)
            .ok_or(WireError::plain(7))
    }
    pub(crate) fn context(
        &self,
        raw: RawContext,
        request: u64,
    ) -> Result<(SessionHandle, CommandContext), WireError> {
        let handle = self.session(raw.session)?;
        let p = self.app.projection(handle)?.editor;
        let interaction = match p.interaction {
            InteractionState::Idle => None,
            InteractionState::Drawing(id) | InteractionState::Transforming(id) => Some(id),
        };
        if p.session.get() != raw.document
            || interaction.map_or(0, InteractionId::get) != raw.interaction
        {
            return Err(WireError::plain(7));
        }
        Ok((
            handle,
            CommandContext {
                session: p.session,
                request: RequestId(request),
                interaction,
            },
        ))
    }
    pub(crate) fn effect(&self, handle: SessionHandle, id: u64) -> Result<EffectToken, WireError> {
        let entry = self
            .sessions
            .iter()
            .find(|s| s.handle == handle)
            .and_then(|s| s.effect);
        match entry {
            Some((known, token))
                if known == id && self.app.projection(handle)?.continuation == Some(token) =>
            {
                Ok(token)
            }
            _ => Err(WireError::plain(7)),
        }
    }
    pub(crate) fn track(
        &mut self,
        handle: SessionHandle,
        p: ApplicationProjection,
        effect_id: u64,
    ) -> u64 {
        let slot = self
            .sessions
            .iter_mut()
            .find(|s| s.handle == handle)
            .expect("resolved session");
        slot.effect = p.continuation.map(|token| {
            let id = slot
                .effect
                .filter(|(_, old)| *old == token)
                .map_or(effect_id, |(id, _)| id);
            (id, token)
        });
        slot.effect.map_or(0, |(id, _)| id)
    }
    pub(crate) fn register_operation(
        &mut self,
        handle: OperationId,
        session: SessionHandle,
        id: u64,
        write: bool,
        active: bool,
    ) -> u64 {
        if let Some(op) = self.operations.iter_mut().find(|o| o.handle == handle) {
            op.active |= active;
            return op.id;
        }
        assert!(
            self.operations.len() < self.limits.operations,
            "issued operations bounded by facade"
        );
        self.operations.push(Op {
            id,
            handle,
            session,
            write,
            active,
        });
        id
    }
}
