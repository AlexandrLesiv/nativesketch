use crate::presenter::Ui;
use adw::prelude::*;
use gtk::{gio, glib};
use sketch_core::application::routing::*;
impl Ui {
    pub(crate) fn choose_file(&self, token: EffectToken, save: bool) {
        let Ok(context) = self.effect_context(token) else {
            return;
        };
        let dialog = gtk::FileDialog::builder()
            .title(if save { "Save Drawing" } else { "Open Drawing" })
            .build();
        if save {
            dialog.set_initial_name(Some("Drawing.nsketch"));
        }
        let ui = self.clone();
        let callback = move |result: Result<gio::File, glib::Error>| match result {
            Ok(file) => match file.path() {
                Some(path) => {
                    let result = {
                        let mut model = ui.model.borrow_mut();
                        ui.provider
                            .borrow_mut()
                            .supply(&mut model.owner, context, token, path)
                    };
                    if let Err(error) = result {
                        crate::diagnostics::record(format_args!(
                            "event=resource_supply_rejected error={error:?}"
                        ));
                    }
                    ui.process_effects();
                    ui.refresh();
                }
                None => {
                    ui.reply(context, token, EffectDecision::Cancel);
                    ui.error(if save {
                        "This location is not a local file. Choose a local folder."
                    } else {
                        "This location is not a local file. Choose a local drawing."
                    });
                }
            },
            Err(error) => {
                ui.reply(context, token, EffectDecision::Cancel);
                if !file_dialog_cancelled(&error) {
                    ui.error("File chooser failed. Please try again.");
                }
            }
        };
        if save {
            dialog.save(Some(&self.window), None::<&gio::Cancellable>, callback);
        } else {
            dialog.open(Some(&self.window), None::<&gio::Cancellable>, callback);
        }
    }
}
fn file_dialog_cancelled(error: &glib::Error) -> bool {
    error.matches(gtk::DialogError::Dismissed)
        || error.matches(gtk::DialogError::Cancelled)
        || error.matches(gio::IOErrorEnum::Cancelled)
}
