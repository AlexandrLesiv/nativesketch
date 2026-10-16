use crate::presenter::Ui;
use adw::prelude::*;
use gtk::{gdk, glib};
use std::{cell::RefCell, rc::Rc};

pub(crate) fn connect(ui: &Ui) {
    let canvas = &ui.canvas;
    let pinch = gtk::GestureZoom::new();
    let drag = gtk::GestureDrag::builder().n_points(2).build();
    drag.group_with(&pinch);
    // Both recognizers use one document anchor, so pan never overwrites pinch offset.
    let navigation = Rc::new(RefCell::new(None::<(f64, f64, f64, f64, f64)>));
    {
        let u = ui.clone();
        let navigation = navigation.clone();
        pinch.connect_begin(move |g, _| {
            let mut m = u.model.borrow_mut();
            m.cancel();
            if let Some((x, y)) = g.bounding_box_center() {
                let p = m.point(x, y);
                *navigation.borrow_mut() = Some((x, y, p.0, p.1, m.zoom));
            }
        });
    }
    {
        let u = ui.clone();
        let navigation = navigation.clone();
        pinch.connect_scale_changed(move |g, scale| {
            if let (Some((_, _, dx, dy, zoom)), Some((x, y))) =
                (*navigation.borrow(), g.bounding_box_center())
            {
                let mut m = u.model.borrow_mut();
                m.zoom = (zoom * scale).clamp(0.05, 32.0);
                m.offset = (x - dx * m.zoom, y - dy * m.zoom);
            }
            u.refresh();
        });
    }
    {
        let u = ui.clone();
        let navigation = navigation.clone();
        drag.connect_drag_begin(move |_, x, y| {
            let mut m = u.model.borrow_mut();
            m.cancel();
            if navigation.borrow().is_none() {
                let p = m.point(x, y);
                *navigation.borrow_mut() = Some((x, y, p.0, p.1, m.zoom));
            }
        });
    }
    {
        let u = ui.clone();
        let navigation = navigation.clone();
        drag.connect_drag_update(move |_, x, y| {
            if let Some((cx, cy, dx, dy, _)) = *navigation.borrow() {
                let mut m = u.model.borrow_mut();
                m.offset = (cx + x - dx * m.zoom, cy + y - dy * m.zoom);
            }
            u.refresh();
        });
    }
    {
        let navigation = navigation.clone();
        pinch.connect_end(move |_, _| *navigation.borrow_mut() = None);
    }
    canvas.add_controller(pinch);
    canvas.add_controller(drag);
    let wheel = gtk::EventControllerScroll::new(gtk::EventControllerScrollFlags::BOTH_AXES);
    {
        let u = ui.clone();
        wheel.connect_scroll(move |g, x, y| {
            u.action(|m| {
                m.cancel();
                if g.current_event_state()
                    .contains(gdk::ModifierType::CONTROL_MASK)
                {
                    m.zoom_to(m.zoom * (-y * 0.08).exp());
                } else {
                    m.offset.0 -= x * 24.0;
                    m.offset.1 -= y * 24.0;
                }
                Ok(())
            });
            glib::Propagation::Stop
        });
    }
    canvas.add_controller(wheel);
}
