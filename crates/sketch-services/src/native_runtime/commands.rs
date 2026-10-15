use super::{NativeRuntime, owner_types::*};
use sketch_core::application::{routing::*, *};
impl NativeRuntime {
    pub fn file_available(&self, window: u64, intent: FileIntent) -> Result<(), RuntimeError> {
        self.check_owner()?;
        if self.stopping {
            return Err(RuntimeError::Stopped);
        }
        if self.barrier.is_some() {
            return Err(RuntimeError::Busy);
        }
        self.capacity()?;
        self.session(window)?
            .app
            .file_availability(intent)
            .map_err(RuntimeError::Application)
    }
    pub fn dispatch(
        &mut self,
        window: u64,
        context: CommandContext,
        command: ApplicationCommand<'_>,
    ) -> Result<ActionResult, RuntimeError> {
        self.check_owner()?;
        if self.stopping {
            return Err(RuntimeError::Stopped);
        }
        let index = self.slot(window)?;
        if self.sessions[index].detached || self.sessions[index].close_confirmed {
            return Err(RuntimeError::Stale);
        }
        let retry = match &command {
            ApplicationCommand::File(intent) => {
                self.file_available(window, *intent)?;
                match intent {
                    FileIntent::Save => 20,
                    FileIntent::SaveAs => 21,
                    FileIntent::ExportCopy => 22,
                    FileIntent::Navigate(NavigationIntent::Open) => 24,
                    _ => 0,
                }
            }
            ApplicationCommand::Edit(_) | ApplicationCommand::Input(_) => 0,
            _ => return Err(RuntimeError::Invalid),
        };
        let clears_recovery = matches!(&command, ApplicationCommand::File(FileIntent::Navigate(intent)) if !matches!(intent, NavigationIntent::Recover(_)));
        let outcome = self.sessions[index]
            .app
            .dispatch(&mut self.coordinator, context, command)
            .map_err(|error| RuntimeError::Application(error.failure))?;
        if clears_recovery {
            self.sessions[index].requested_recovery = None;
        }
        self.action(window, outcome, self.sessions[index].bound_grant, retry)
    }
    pub fn reply(
        &mut self,
        window: u64,
        context: CommandContext,
        token: EffectToken,
        decision: EffectDecision,
    ) -> Result<ActionResult, RuntimeError> {
        self.check_owner()?;
        if matches!(
            decision,
            EffectDecision::Destination(_) | EffectDecision::Source(_)
        ) {
            return Err(RuntimeError::Invalid);
        }
        if !matches!(decision, EffectDecision::Cancel) {
            if self.barrier.is_some() {
                return Err(RuntimeError::Busy);
            }
            self.capacity()?;
        }
        let index = self.slot(window)?;
        let outcome = self.sessions[index]
            .app
            .dispatch(
                &mut self.coordinator,
                context,
                ApplicationCommand::ResolveEffect(EffectReply { token, decision }),
            )
            .map_err(|error| RuntimeError::Application(error.failure))?;
        if matches!(decision, EffectDecision::Cancel) {
            self.sessions[index].requested_recovery = None;
            for operation in &mut self.operations {
                if operation.window == window
                    && matches!(operation.kind,
                    OperationKind::Resolve { token: held, .. } | OperationKind::Prepare(held) if held == token)
                {
                    operation.cancelled = true;
                }
            }
        }
        self.action(window, outcome, self.sessions[index].bound_grant, 0)
    }
    pub(super) fn action(
        &mut self,
        window: u64,
        mut outcome: ApplicationOutcome,
        grant: u64,
        retry: u32,
    ) -> Result<ActionResult, RuntimeError> {
        let index = self.slot(window)?;
        let state = outcome.projection.editor;
        if outcome.changes.document {
            self.sessions[index].pristine = false;
            self.sessions[index].schedule.changed(
                state.session.get(),
                state.revision.0,
                self.started.elapsed().as_millis() as u64,
            );
        }
        if outcome.changes.session_replaced {
            self.sessions[index].recovery = None;
            self.sessions[index].last_checkpoint = None;
            self.sessions[index].schedule.replace_session();
        }
        let (effect, operation) =
            self.capture_effect(window, outcome.effect.take(), grant, retry)?;
        if matches!(effect, IntentEffect::CloseConfirmed) {
            self.sessions[index].close_confirmed = true;
        }
        if let Some(retired) = outcome.retired.take() {
            self.retire(super::work::Retired::Editor(retired))?;
        }
        Ok(ActionResult {
            outcome,
            effect,
            operation,
            retry_action: if matches!(effect, IntentEffect::Failure(_)) {
                retry
            } else {
                0
            },
        })
    }
}
