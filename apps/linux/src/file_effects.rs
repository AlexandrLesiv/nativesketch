use crate::presenter::Ui;
use adw::prelude::*;
use sketch_app::{HostEffect, Intent};
use sketch_core::application::{CommandContext, routing::*};
impl Ui {
    pub(crate) fn file(&self, intent: FileIntent) {
        if self.closing.get() {
            return;
        }
        let result = {
            let mut model = self.model.borrow_mut();
            let result = model.owner.dispatch(Intent::File(intent));
            if result.is_ok() {
                model.gesture = crate::model::Gesture::Idle;
            }
            result
        };
        if let Err(error) = result {
            crate::diagnostics::record(format_args!("event=file_intent_rejected error={error:?}"));
            self.error(&format!(
                "The operation failed: {error:?}. Your drawing was not replaced."
            ));
        }
        self.process_effects();
        self.refresh();
    }
    pub(crate) fn reply(
        &self,
        context: CommandContext,
        token: EffectToken,
        decision: EffectDecision,
    ) {
        let result = self
            .model
            .borrow_mut()
            .owner
            .reply(context, token, decision);
        if let Err(error) = result {
            crate::diagnostics::record(format_args!("event=dialog_reply_rejected error={error:?}"));
        }
        self.process_effects();
        self.refresh();
    }
    pub(crate) fn process_effects(&self) {
        loop {
            let effect = self.model.borrow_mut().owner.effects.pop_front();
            let Some(effect) = effect else {
                break;
            };
            match effect {
                HostEffect::ChooseDestination { token, .. } => self.choose_file(token, true),
                HostEffect::ChooseSource(token) => self.choose_file(token, false),
                HostEffect::ConfirmUnsaved { token, .. } => self.confirm_unsaved(token),
                HostEffect::NavigationReady(token) => {
                    let context = self.effect_context(token);
                    if let Ok(context) = context {
                        self.reply(context, token, EffectDecision::ConfirmClose);
                    }
                }
                HostEffect::CloseConfirmed => {
                    let result = {
                        let mut model = self.model.borrow_mut();
                        let session = model.owner.session;
                        model.owner.app.detach_session(session)
                    };
                    if result.is_ok() {
                        self.closing.set(true);
                        self.window.set_visible(false);
                    } else {
                        self.error("Could not close while document work is pending.");
                    }
                }
                HostEffect::AwaitWrites(token) => self.awaiting.set(Some(token)),
                HostEffect::Failure(error) => self.error(&format!(
                    "The operation failed: {error:?}. Your drawing was not replaced."
                )),
                HostEffect::WorkQueued(_) | HostEffect::Cancelled => {}
            }
        }
    }
    pub(crate) fn effect_context(
        &self,
        token: EffectToken,
    ) -> Result<CommandContext, sketch_app::AppError> {
        let mut model = self.model.borrow_mut();
        let mut context = model.owner.context()?;
        // Preserve the issued session/request; never retarget a late dialog to a new document.
        context.session = token.session();
        context.request = token.request();
        Ok(context)
    }
    fn confirm_unsaved(&self, token: EffectToken) {
        let Ok(context) = self.effect_context(token) else {
            return;
        };
        let dialog = adw::AlertDialog::builder()
            .heading("Save changes?")
            .body("Save your drawing before continuing, or discard the unsaved changes.")
            .build();
        dialog.add_responses(&[
            ("cancel", "Cancel"),
            ("discard", "Discard"),
            ("save", "Save…"),
        ]);
        dialog.set_default_response(Some("save"));
        dialog.set_close_response("cancel");
        dialog.set_response_appearance("discard", adw::ResponseAppearance::Destructive);
        let ui = self.clone();
        dialog.connect_response(None, move |_, response| {
            let choice = match response {
                "discard" => UnsavedChoice::Discard,
                "save" => UnsavedChoice::Save,
                _ => UnsavedChoice::Cancel,
            };
            ui.reply(context, token, EffectDecision::Unsaved(choice));
        });
        dialog.present(Some(&self.window));
    }
}
