use super::{NativeRuntime, owner_types::*, work::*};
impl NativeRuntime {
    pub fn retry_recovery(&mut self, request: u64, window: u64) -> Result<u64, RuntimeError> {
        if window != 0 {
            self.session(window)?;
        }
        let registration = if window == 0 {
            None
        } else {
            self.session(window)?
                .recovery
                .as_ref()
                .filter(|r| !r.approved)
                .map(|r| r.identity)
        };
        let work = registration.map_or(Work::Retry, Work::Register);
        let id = self.maintenance(request, window, work)?;
        self.operations
            .iter_mut()
            .find(|op| op.id == id)
            .expect("issued retry")
            .kind = if registration.is_some() {
            OperationKind::Register
        } else {
            OperationKind::RetryRecovery
        };
        for slot in &mut self.sessions {
            if (window == 0 || slot.window == window)
                && let Some(recovery) = &mut slot.recovery
            {
                recovery.state = 1;
            }
        }
        Ok(id)
    }
}

#[cfg(feature = "native-test-controls")]
impl NativeRuntime {
    pub fn arm_test(&mut self, control: super::TestControl) -> Result<(), RuntimeError> {
        self.maintenance(0, 0, Work::ArmTest(control)).map(|_| ())
    }
}
