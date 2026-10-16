use crate::model::Model;
use adw::prelude::*;
use std::{cell::RefCell, rc::Rc};

pub(crate) type Shared = Rc<RefCell<Model>>;
#[derive(Clone)]
pub(crate) struct Ui {
    pub(crate) model: Shared,
    pub(crate) window: adw::ApplicationWindow,
    pub(crate) canvas: gtk::DrawingArea,
    pub(crate) status: gtk::Label,
    pub(crate) undo: gtk::Button,
    pub(crate) redo: gtk::Button,
    pub(crate) selected: Vec<gtk::Widget>,
    pub(crate) cycling: Vec<gtk::Widget>,
}
impl Ui {
    pub(crate) fn refresh(&self) {
        let m = self.model.borrow();
        let s = m.editor.state();
        self.undo.set_sensitive(s.can_undo);
        self.redo.set_sensitive(s.can_redo);
        for w in &self.selected {
            w.set_sensitive(s.selected != 0);
        }
        for w in &self.cycling {
            w.set_sensitive(s.count > 0);
        }
        let text = format!(
            "{} strokes · {:.0}%{}{}",
            s.count,
            m.zoom * 100.0,
            if s.dirty { " · Unsaved" } else { "" },
            if s.selected != 0 {
                " · Stroke selected"
            } else {
                ""
            }
        );
        self.status.set_text(&text);
        self.canvas
            .update_property(&[gtk::accessible::Property::Label(&format!(
                "Drawing canvas, {text}"
            ))]);
        self.window.set_title(Some(if s.dirty {
            "Native Sketch — Unsaved"
        } else {
            "Native Sketch"
        }));
        self.canvas.queue_draw();
    }
    pub(crate) fn action(&self, f: impl FnOnce(&mut Model) -> sketch_core::Result<()>) {
        let result = {
            let mut m = self.model.borrow_mut();
            m.cancel();
            f(&mut m)
        };
        if let Err(e) = result {
            self.model.borrow_mut().cancel();
            self.error(&format!(
                "The operation failed: {e:?}. Your drawing was not replaced."
            ));
        }
        self.refresh();
    }
    pub(crate) fn error(&self, text: &str) {
        let dialog = adw::AlertDialog::builder()
            .heading("Could not complete action")
            .body(text)
            .build();
        dialog.add_response("ok", "OK");
        dialog.present(Some(&self.window));
    }
}
