use crate::{controls::Controls, presenter::Ui};
use adw::prelude::*;
use gtk::{gdk, glib};
use sketch_app::Intent;
pub(crate) fn connect(ui: &Ui, _: &Controls) {
    let keys = gtk::EventControllerKey::new();
    let input = ui.clone();
    keys.connect_key_pressed(move |_, key, _, mods| {
        let ctrl = mods.intersects(gdk::ModifierType::CONTROL_MASK | gdk::ModifierType::META_MASK);
        if key == gdk::Key::space {
            input.model.borrow_mut().space = true;
            return glib::Propagation::Stop;
        }
        match crate::shortcut_intent::translate(
            key,
            ctrl,
            mods.contains(gdk::ModifierType::SHIFT_MASK),
        ) {
            Some(Intent::File(file)) => {
                input.file(file);
                glib::Propagation::Stop
            }
            Some(intent) => {
                input.action(|model| model.owner.dispatch(intent));
                glib::Propagation::Stop
            }
            None => glib::Propagation::Proceed,
        }
    });
    let released = ui.model.clone();
    keys.connect_key_released(move |_, key, _, _| {
        if key == gdk::Key::space {
            released.borrow_mut().space = false;
        }
    });
    // Attached only to the canvas: GTK Entry/SpinButton keep their native shortcuts.
    ui.canvas.add_controller(keys);
    let focus = gtk::EventControllerFocus::new();
    let input = ui.clone();
    focus.connect_leave(move |_| {
        let mut model = input.model.borrow_mut();
        model.space = false;
        model.cancel();
        drop(model);
        input.refresh();
    });
    ui.canvas.add_controller(focus);
}
