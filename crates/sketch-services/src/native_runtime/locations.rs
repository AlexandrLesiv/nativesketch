use super::{NativeRuntime, owner_types::*, work::*};
use sketch_core::application::{routing::*, *};
use std::path::PathBuf;
impl NativeRuntime {
    pub fn supply_location(
        &mut self,
        window: u64,
        context: CommandContext,
        token: EffectToken,
        grant: u64,
        path: PathBuf,
    ) -> Result<u64, RuntimeError> {
        self.check_owner()?;
        if self.stopping {
            return Err(RuntimeError::Stopped);
        }
        if self.barrier.is_some() {
            return Err(RuntimeError::Busy);
        }
        self.capacity()?;
        let projection = self.session(window)?.app.projection();
        if projection.continuation != Some(token)
            || projection.editor.session != context.session
            || token.session() != context.session
        {
            return Err(RuntimeError::Stale);
        }
        if !path.is_absolute() {
            return Err(RuntimeError::Invalid);
        }
        let source = match projection.file {
            FilePhase::Source => true,
            FilePhase::Destination => false,
            _ => return Err(RuntimeError::Invalid),
        };
        self.new_grant(grant)?;
        let id = self.enqueue(
            OperationOwner {
                window,
                session: context.session.get(),
                request: context.request.0,
                grant,
            },
            OperationKind::Resolve { token, source },
            OperationState::Queued(Box::new(Work::Resolve(path))),
            if source { 24 } else { 21 },
        )?;
        self.release_grant(grant)?;
        self.barrier = Some(id);
        self.pump()?;
        Ok(id)
    }
}
