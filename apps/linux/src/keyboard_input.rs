use crate::{controls::Controls, presenter::Ui};
use adw::prelude::*;
use gtk::{gdk, glib};
use std::rc::Rc;

pub(crate) fn connect(ui: &Ui, controls: &Controls) {
    let canvas = &ui.canvas;
    let model = ui.model.clone();
    let pen = &controls.pen;
    let select = &controls.select;
    let keys = gtk::EventControllerKey::new();
    {
        let u = ui.clone();
        let pen = pen.clone();
        let select = select.clone();
        keys.connect_key_pressed(move |_, key, _, mods| {
            let ctrl =
                mods.intersects(gdk::ModifierType::CONTROL_MASK | gdk::ModifierType::META_MASK);
            if key == gdk::Key::space {
                u.model.borrow_mut().space = true;
                return glib::Propagation::Stop;
            }
            if !ctrl && key == gdk::Key::p {
                pen.set_active(true);
                return glib::Propagation::Stop;
            }
            if !ctrl && key == gdk::Key::v {
                select.set_active(true);
                return glib::Propagation::Stop;
            }
            if ctrl && key == gdk::Key::s {
                u.save(None);
                return glib::Propagation::Stop;
            }
            if ctrl && key == gdk::Key::o {
                u.open();
                return glib::Propagation::Stop;
            }
            if ctrl && key == gdk::Key::n {
                let inner = u.clone();
                u.guard(Rc::new(move || inner.action(|m| m.editor.clear())));
                return glib::Propagation::Stop;
            }
            let op = if key == gdk::Key::Escape {
                Some(0)
            } else if ctrl && (key == gdk::Key::z || key == gdk::Key::Z) {
                Some(if mods.contains(gdk::ModifierType::SHIFT_MASK) {
                    2
                } else {
                    1
                })
            } else if key == gdk::Key::Delete || key == gdk::Key::BackSpace {
                Some(3)
            } else if key == gdk::Key::p {
                Some(4)
            } else if key == gdk::Key::v {
                Some(5)
            } else {
                None
            };
            if let Some(op) = op {
                u.action(|m| {
                    let s = m.editor.state();
                    match op {
                        1 if s.can_undo => m.editor.undo()?,
                        2 if s.can_redo => m.editor.redo()?,
                        3 if s.selected != 0 => m.editor.delete_selected()?,
                        4 => m.select = false,
                        5 => m.select = true,
                        _ => {}
                    }
                    Ok(())
                });
                glib::Propagation::Stop
            } else {
                glib::Propagation::Proceed
            }
        });
    }
    {
        let m = model;
        keys.connect_key_released(move |_, key, _, _| {
            if key == gdk::Key::space {
                m.borrow_mut().space = false;
            }
        });
    }
    canvas.add_controller(keys);
    let focus = gtk::EventControllerFocus::new();
    {
        let u = ui.clone();
        focus.connect_leave(move |_| {
            let mut m = u.model.borrow_mut();
            m.space = false;
            m.cancel();
            drop(m);
            u.refresh();
        });
    }
    canvas.add_controller(focus);
}
