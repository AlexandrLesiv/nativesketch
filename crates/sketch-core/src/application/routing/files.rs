use super::*;
impl ApplicationSession {
    /// Advisory native action state; dispatch repeats this preflight before mutation.
    pub fn file_availability(&self, intent: FileIntent) -> Result<(), ApplicationFailure> {
        let state = self.editor.view_state();
        let capabilities = state.capabilities;
        let allowed = match intent {
            FileIntent::ExportCopy => capabilities.export_document,
            FileIntent::Checkpoint(_) => capabilities.edit_document,
            FileIntent::Navigate(NavigationIntent::Close) => true,
            FileIntent::Navigate(_) => capabilities.edit_document,
            _ => capabilities.edit_document && capabilities.export_document,
        };
        if !allowed {
            return Err(ApplicationFailure::CapabilityDenied);
        }
        // A queued Save As is not an active chooser. Ordinary Save still uses the old binding.
        let bound_save =
            intent == FileIntent::Save && self.editor.save_state().destination().is_some();
        if self.pending.is_some() && !bound_save {
            return Err(ApplicationFailure::Busy);
        }
        if (intent == FileIntent::SaveAs || intent == FileIntent::Save && !bound_save)
            && self.editor.save_state().binding_pending()
        {
            return Err(ApplicationFailure::Busy);
        }
        if matches!(intent, FileIntent::Checkpoint(_))
            && state.interaction != InteractionState::Idle
        {
            return Err(ApplicationFailure::Busy);
        }
        Ok(())
    }
    pub(super) fn file(
        &mut self,
        queue: &mut SaveCoordinator,
        context: CommandContext,
        intent: FileIntent,
    ) -> Result<ApplicationEffect, ApplicationFailure> {
        self.file_availability(intent)?;
        let bound_save =
            intent == FileIntent::Save && self.editor.save_state().destination().is_some();
        let token = self.issue(context.request)?;
        // Only accepted explicit file intents cancel temporary interaction; checkpoints defer.
        self.cancel_interaction(context)?;
        Ok(match intent {
            FileIntent::Navigate(intent) => {
                self.navigation = Some(Navigation { intent, save: None });
                self.advance(queue, token, false)
            }
            FileIntent::Checkpoint(destination) => {
                self.admit(queue, context.request, SavePurpose::Checkpoint, destination)
            }
            FileIntent::Save if bound_save => self.admit(
                queue,
                context.request,
                SavePurpose::Save,
                self.editor.save_state().destination().unwrap(),
            ),
            intent => {
                let purpose = if intent == FileIntent::ExportCopy {
                    SavePurpose::ExportCopy
                } else {
                    SavePurpose::SaveAs
                };
                self.pending = Some(Pending::Destination(token, purpose));
                ApplicationEffect::ChooseDestination { token, purpose }
            }
        })
    }
    pub(super) fn admit(
        &mut self,
        queue: &mut SaveCoordinator,
        request: RequestId,
        purpose: SavePurpose,
        destination: DestinationId,
    ) -> ApplicationEffect {
        if self.saves.len() == 1024 {
            return ApplicationEffect::Failure(ApplicationFailure::Busy);
        }
        match queue.request(&mut self.editor, request, purpose, destination) {
            Ok(token) => {
                self.saves.push(token);
                ApplicationEffect::SaveQueued(token)
            }
            Err(error) => ApplicationEffect::Failure(ApplicationFailure::Storage(error)),
        }
    }
    pub(super) fn save_for_navigation(
        &mut self,
        queue: &mut SaveCoordinator,
        token: EffectToken,
    ) -> ApplicationEffect {
        if let Some(destination) = self.editor.save_state().destination() {
            let effect = self.admit(queue, token.request, SavePurpose::Save, destination);
            self.wait_for_save(token, &effect);
            effect
        } else {
            self.pending = Some(Pending::Destination(token, SavePurpose::SaveAs));
            ApplicationEffect::ChooseDestination {
                token,
                purpose: SavePurpose::SaveAs,
            }
        }
    }
    pub(super) fn wait_for_save(&mut self, token: EffectToken, effect: &ApplicationEffect) {
        if let ApplicationEffect::SaveQueued(save) = effect {
            self.navigation.as_mut().unwrap().save = Some(*save);
            self.pending = Some(Pending::Writes(token));
        } else {
            self.navigation = None;
            self.pending = None;
        }
    }
}
