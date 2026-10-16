use crate::model::Model;
use adw::prelude::*;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

pub(crate) type Shared = Rc<RefCell<Model>>;
#[derive(Clone)]
pub(crate) struct Ui {
    pub(crate) model: Shared,
    pub(crate) provider: Rc<RefCell<crate::provider::Provider>>,
    pub(crate) closing: Rc<Cell<bool>>,
    pub(crate) awaiting: Rc<Cell<Option<sketch_core::application::routing::EffectToken>>>,
    pub(crate) window: adw::ApplicationWindow,
    pub(crate) canvas: gtk::DrawingArea,
    pub(crate) status: gtk::Label,
    pub(crate) pen: gtk::ToggleButton,
    pub(crate) select: gtk::ToggleButton,
    pub(crate) undo: gtk::Button,
    pub(crate) redo: gtk::Button,
    pub(crate) selected: Vec<gtk::Widget>,
    pub(crate) cycling: Vec<gtk::Widget>,
}
impl Ui {
    pub(crate) fn refresh(&self) {
        if self.closing.get() {
            return;
        }
        let m = self.model.borrow();
        let s = m.owner.state();
        let projection = m.owner.app.projection(m.owner.session).unwrap().editor;
        let tool = projection.tool;
        let available = |action| {
            projection.action_availability(action)
                == sketch_core::application::Availability::Enabled
        };
        let zoom = m.zoom;
        drop(m);
        self.pen
            .set_active(tool == sketch_core::application::ToolId::Pen);
        self.select
            .set_active(tool == sketch_core::application::ToolId::Select);
        self.undo
            .set_sensitive(available(sketch_core::application::EditorAction::Undo));
        self.redo
            .set_sensitive(available(sketch_core::application::EditorAction::Redo));
        for w in &self.selected {
            w.set_sensitive(available(
                sketch_core::application::EditorAction::DeleteSelection,
            ));
        }
        for w in &self.cycling {
            w.set_sensitive(available(
                sketch_core::application::EditorAction::CycleSelection(
                    sketch_core::application::CycleDirection::Next,
                ),
            ));
        }
        let text = format!(
            "{} strokes · {:.0}%{}{}",
            s.count,
            zoom * 100.0,
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
    pub(crate) fn action(&self, f: impl FnOnce(&mut Model) -> Result<(), sketch_app::AppError>) {
        if self.closing.get() {
            return;
        }
        let result = {
            let mut m = self.model.borrow_mut();
            let result = f(&mut m);
            if m.owner.state().interaction == 0 {
                m.gesture = crate::model::Gesture::Idle;
            }
            result
        };
        if let Err(e) = result {
            crate::diagnostics::record(format_args!("event=action_rejected error={e:?}"));
            self.error(&format!(
                "The operation failed: {e:?}. Your drawing was not replaced."
            ));
        }
        self.process_effects();
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
