use super::{NativeRuntime, owner_types::*, work::*};
impl NativeRuntime {
    pub fn retry_recovery(&mut self, request: u64, window: u64) -> Result<u64, RuntimeError> {
        self.check_owner()?;
        self.capacity()?;
        if self.barrier.is_some() || self.stopping {
            return Err(RuntimeError::Busy);
        }
        let (session, identity, approved) = if window == 0 {
            (0, None, true)
        } else {
            let slot = self.session(window)?;
            let registration = slot.recovery.as_ref().ok_or(RuntimeError::Invalid)?;
            (
                slot.app.projection().editor.session.get(),
                Some(registration.identity),
                registration.approved,
            )
        };
        let (kind, work) = if approved {
            (OperationKind::RetryRecovery(identity), Work::Retry)
        } else {
            let identity = identity.expect("scoped registration");
            (OperationKind::Register(identity), Work::Register(identity))
        };
        let id = self.enqueue(
            OperationOwner {
                window,
                session,
                request,
                grant: 0,
            },
            kind,
            OperationState::Queued(Box::new(work)),
            0,
        )?;
        for slot in &mut self.sessions {
            if (window == 0 || slot.window == window)
                && let Some(recovery) = &mut slot.recovery
                && (window != 0 || recovery.approved)
            {
                recovery.state = 1;
            }
        }
        self.pump()?;
        Ok(id)
    }
}

#[cfg(feature = "native-test-controls")]
impl NativeRuntime {
    pub fn arm_test(&mut self, control: super::TestControl) -> Result<(), RuntimeError> {
        self.maintenance(0, 0, Work::ArmTest(control)).map(|_| ())
    }
}
