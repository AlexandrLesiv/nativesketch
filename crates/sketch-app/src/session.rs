use crate::{
    operations::{Operation, Phase},
    owner::Slot,
    work::Retired,
    *,
};
impl PortableApp {
    pub fn create_session(&mut self, config: SessionConfig) -> Result<SessionHandle, AppError> {
        if self.sessions.len() >= self.limits.sessions {
            return Err(AppError::Capacity);
        }
        let handle = SessionHandle(crate::ids::reserve(&mut self.next_session)?, self.identity);
        let pristine = true;
        let app = ApplicationSession::new(config.capabilities, config.tool, config.brush, None)?;
        self.sessions.push(Slot {
            handle,
            app,
            bound_resource: None,
            choice: None,
            pristine,
            detached: false,
            close_confirmed: false,
        });
        Ok(handle)
    }
    pub fn projection(&self, handle: SessionHandle) -> Result<ApplicationProjection, AppError> {
        Ok(self.sessions[self.slot(handle)?].app.projection())
    }
    pub fn availability(
        &self,
        handle: SessionHandle,
        action: EditorAction,
    ) -> Result<Availability, AppError> {
        Ok(self.sessions[self.attached(handle)?]
            .app
            .projection()
            .editor
            .action_availability(action))
    }
    pub fn selected_bounds(&self, handle: SessionHandle) -> Result<sketch_core::Bounds, AppError> {
        let QueryResult::SelectedBounds(bounds) = self.sessions[self.attached(handle)?]
            .app
            .query(EditorQuery::SelectedBounds)?
        else {
            unreachable!("query variant")
        };
        Ok(bounds)
    }
    pub fn scene(
        &self,
        handle: SessionHandle,
        cache: &mut sketch_core::SceneCache,
        viewport: sketch_core::RenderViewport,
    ) -> Result<sketch_core::RenderScene, AppError> {
        self.sessions[self.attached(handle)?]
            .app
            .render_scene(cache, viewport)
            .map_err(AppError::Scene)
    }
    pub fn detach_session(&mut self, handle: SessionHandle) -> Result<(), AppError> {
        let index = self.slot(handle)?;
        let slot = &self.sessions[index];
        let state = slot.app.projection().editor;
        if state.interaction != InteractionState::Idle {
            return Err(AppError::Busy);
        }
        if !slot.pristine && !slot.close_confirmed && state.capabilities.edit_document {
            return Err(AppError::CapabilityDenied);
        }
        if !self.operations.iter().any(|op| op.session == handle) {
            let id = self.reserve_operation()?;
            let slot = self.sessions.swap_remove(index);
            let retired = RetiredState {
                stamp: None,
                states: vec![Retired::App(slot.app)],
            };
            self.operations.push(Operation {
                id,
                session: handle,
                request: RequestId(0),
                resource: None,
                save: None,
                continuation: None,
                phase: Phase::Retire(retired),
            });
        } else {
            self.sessions[index].detached = true;
        }
        self.release_unused();
        Ok(())
    }
    pub(crate) fn retire_detached(&mut self, handle: SessionHandle, free_id: OperationId) {
        let Ok(index) = self.slot(handle) else {
            return;
        };
        if !self.sessions[index].detached {
            return;
        }
        if self
            .operations
            .iter()
            .any(|op| op.session == handle && !matches!(op.phase, Phase::Retire(_)))
        {
            return;
        }
        let slot = self.sessions.swap_remove(index);
        if let Some(operation) = self
            .operations
            .iter_mut()
            .find(|op| op.session == handle && matches!(op.phase, Phase::Retire(_)))
        {
            let Phase::Retire(retired) = &mut operation.phase else {
                unreachable!()
            };
            retired.states.push(Retired::App(slot.app));
        } else {
            self.operations.push(Operation {
                id: free_id,
                session: handle,
                request: RequestId(0),
                resource: None,
                save: None,
                continuation: None,
                phase: Phase::Retire(RetiredState {
                    stamp: None,
                    states: vec![Retired::App(slot.app)],
                }),
            });
        }
    }
}
