use sketch_app::{AppError, AppOutcome, Intent, PortableApp, SessionConfig, SessionHandle};
use sketch_core::application::{routing::*, *};
use std::collections::VecDeque;
pub(crate) struct CommandOwner {
    pub app: PortableApp,
    pub session: SessionHandle,
    pub effects: VecDeque<sketch_app::HostEffect>,
    request: u64,
}
impl CommandOwner {
    pub fn new() -> Self {
        let mut app = PortableApp::new(Default::default()).expect("valid host limits");
        let session = app
            .create_session_with_save_policy(
                SessionConfig {
                    capabilities: Capabilities {
                        edit_document: true,
                        export_document: true,
                    },
                    tool: ToolId::Pen,
                    brush: sketch_core::Brush {
                        rgba: [0.14, 0.39, 0.92, 1.0],
                        width: 4.0,
                    }
                    .try_into()
                    .unwrap(),
                },
                SaveDestinationPolicy::AlwaysChoose,
            )
            .expect("initial session");
        Self {
            app,
            session,
            effects: VecDeque::new(),
            request: 0,
        }
    }
    pub fn context(&mut self) -> Result<CommandContext, AppError> {
        self.request = self.request.checked_add(1).ok_or(AppError::Capacity)?;
        let state = self.app.projection(self.session)?.editor;
        Ok(CommandContext {
            session: state.session,
            request: RequestId(self.request),
            interaction: match state.interaction {
                InteractionState::Idle => None,
                InteractionState::Drawing(id) | InteractionState::Transforming(id) => Some(id),
            },
        })
    }
    pub fn dispatch(&mut self, intent: Intent<'_>) -> Result<(), AppError> {
        if self.effects.len() >= 32 {
            return Err(AppError::Capacity);
        }
        let context = self.context()?;
        let outcome = self.app.dispatch(self.session, context, intent)?;
        self.accept(outcome);
        Ok(())
    }
    pub fn accept(&mut self, outcome: AppOutcome) {
        if let Some(save) = outcome.save {
            use sketch_core::application::storage::SaveResult;
            let failure = match save.result {
                SaveResult::Failed(error)
                | SaveResult::Uncertain(error)
                | SaveResult::FailedAfterCancellation(error)
                | SaveResult::UncertainAfterCancellation(error) => Some(error),
                _ => None,
            };
            if let Some(error) = failure {
                self.effects
                    .push_back(sketch_app::HostEffect::Failure(ApplicationFailure::Write(
                        error,
                    )));
            }
        }
        if let Some(effect) = outcome.effect {
            self.effects.push_back(effect);
        }
    }
    pub fn state(&self) -> sketch_core::application::EditorSummary {
        self.app
            .projection(self.session)
            .expect("attached editor")
            .editor
            .document
    }
    pub fn reply(
        &mut self,
        context: CommandContext,
        token: EffectToken,
        decision: EffectDecision,
    ) -> Result<(), AppError> {
        let outcome = self.app.reply(self.session, context, token, decision)?;
        self.accept(outcome);
        Ok(())
    }
}
