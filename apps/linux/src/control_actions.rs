use crate::{controls::Controls, presenter::Ui};
use adw::prelude::*;
use gtk::glib;
use std::rc::Rc;

pub(crate) fn connect(ui: &Ui, controls: &Controls) {
    let model = ui.model.clone();
    let window = ui.window.clone();
    let Controls {
        new,
        open,
        save,
        undo,
        redo,
        pen,
        select,
        color,
        width,
        previous,
        next,
        scale,
        apply,
        delete,
        out,
        reset,
        zoom_in,
    } = controls.clone();
    for (b, op) in [
        (undo, 0),
        (redo, 1),
        (previous, 2),
        (next, 3),
        (delete, 4),
        (out, 5),
        (reset, 6),
        (zoom_in, 7),
    ] {
        let u = ui.clone();
        let select = select.clone();
        b.connect_clicked(move |_| {
            if op == 2 || op == 3 {
                select.set_active(true);
            }
            u.action(|m| {
                match op {
                    0 => m.editor.undo()?,
                    1 => m.editor.redo()?,
                    2 => m.editor.select_cycle(-1)?,
                    3 => m.editor.select_cycle(1)?,
                    4 => m.editor.delete_selected()?,
                    5 => m.zoom_to(m.zoom / 1.25),
                    6 => m.zoom_to(1.0),
                    7 => m.zoom_to(m.zoom * 1.25),
                    _ => {}
                }
                Ok(())
            })
        });
    }
    {
        let u = ui.clone();
        apply.connect_clicked(move |_| {
            u.action(|m| {
                let b = m.editor.state().bounds;
                m.editor.resize_begin()?;
                m.editor.resize_update(
                    scale.value() / 100.0,
                    b.x + b.width / 2.0,
                    b.y + b.height / 2.0,
                )?;
                m.editor.resize_commit()
            })
        });
    }
    {
        let u = ui.clone();
        pen.connect_toggled(move |p| {
            if p.is_active() {
                u.action(|m| {
                    m.select = false;
                    m.editor.select_clear()
                });
            }
        });
    }
    {
        let u = ui.clone();
        select.connect_toggled(move |p| {
            if p.is_active() {
                u.action(|m| {
                    m.select = true;
                    Ok(())
                });
            }
        });
    }
    {
        let m = model.clone();
        color.connect_rgba_notify(move |c| {
            let c = c.rgba();
            m.borrow_mut().brush.rgba = [
                c.red() as f64,
                c.green() as f64,
                c.blue() as f64,
                c.alpha() as f64,
            ];
        });
    }
    {
        let m = model.clone();
        width.connect_value_changed(move |w| m.borrow_mut().brush.width = w.value());
    }
    {
        let u = ui.clone();
        save.connect_clicked(move |_| u.save(None));
    }
    {
        let u = ui.clone();
        open.connect_clicked(move |_| u.open());
    }
    {
        let u = ui.clone();
        new.connect_clicked(move |_| {
            let c = u.clone();
            u.guard(Rc::new(move || c.action(|m| m.editor.clear())));
        });
    }
    {
        let u = ui.clone();
        window.connect_close_request(move |_| {
            let w = u.window.clone();
            u.guard(Rc::new(move || w.destroy()));
            glib::Propagation::Stop
        });
    }
    {
        let u = ui.clone();
        window.connect_is_active_notify(move |w| {
            if !w.is_active() {
                u.model.borrow_mut().cancel();
                u.refresh();
            }
        });
    }
}
