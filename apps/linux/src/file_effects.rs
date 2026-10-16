use crate::presenter::Ui;
use adw::prelude::*;
use gtk::{gio, glib};
use std::rc::Rc;

impl Ui {
    pub(crate) fn save(&self, then: Option<Rc<dyn Fn()>>) {
        self.model.borrow_mut().cancel();
        let dialog = gtk::FileDialog::builder()
            .title("Save Drawing")
            .initial_name("Drawing.nsketch")
            .build();
        let ui = self.clone();
        dialog.save(
            Some(&self.window),
            None::<&gio::Cancellable>,
            move |result| {
                let file = match result {
                    Ok(file) => file,
                    Err(error) => {
                        if !file_dialog_cancelled(&error) {
                            ui.error(&format!("File chooser failed: {error}"));
                        }
                        return;
                    }
                };
                {
                    if let Some(path) = file.path() {
                        let result = ui.model.borrow_mut().editor.save(&path);
                        match result {
                            Ok(()) => {
                                ui.refresh();
                                if let Some(f) = then {
                                    f();
                                }
                            }
                            Err(e) => ui.error(&format!(
                                "Save failed: {e:?}. The drawing remains unsaved."
                            )),
                        }
                    } else {
                        ui.error("This location is not a local file. Choose a local folder.");
                    }
                }
            },
        );
    }
    pub(crate) fn guard(&self, then: Rc<dyn Fn()>) {
        self.model.borrow_mut().cancel();
        if !self.model.borrow().editor.state().dirty {
            then();
            return;
        }
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
        dialog.connect_response(None, move |_, response| match response {
            "discard" => then(),
            "save" => ui.save(Some(then.clone())),
            _ => {}
        });
        dialog.present(Some(&self.window));
    }
    pub(crate) fn open(&self) {
        let ui = self.clone();
        self.guard(Rc::new(move || {
            let dialog = gtk::FileDialog::builder().title("Open Drawing").build();
            let inner = ui.clone();
            dialog.open(Some(&ui.window), None::<&gio::Cancellable>, move |result| {
                let file = match result {
                    Ok(file) => file,
                    Err(error) => {
                        if !file_dialog_cancelled(&error) {
                            inner.error(&format!("File chooser failed: {error}"));
                        }
                        return;
                    }
                };
                {
                    if let Some(path) = file.path() {
                        inner.action(|m| m.editor.load(&path));
                    } else {
                        inner.error("This location is not a local file. Choose a local drawing.");
                    }
                }
            });
        }));
    }
}
fn file_dialog_cancelled(error: &glib::Error) -> bool {
    error.matches(gtk::DialogError::Dismissed)
        || error.matches(gtk::DialogError::Cancelled)
        || error.matches(gio::IOErrorEnum::Cancelled)
}
