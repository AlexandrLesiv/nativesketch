use crate::{model::Gesture, presenter::Ui};
use adw::prelude::*;
use glib::translate::IntoGlib;
use gtk::{gdk, glib};

pub(crate) fn connect(ui: &Ui) {
    let canvas = &ui.canvas;
    let stylus = gtk::GestureStylus::new();
    stylus.set_stylus_only(false);
    {
        let u = ui.clone();
        stylus.connect_down(move |g, x, y| {
            u.canvas.grab_focus();
            let pressure = g.device_tool().and_then(|_| g.axis(gdk::AxisUse::Pressure));
            let device = if g.device_tool().is_some() {
                1
            } else if g
                .current_event_device()
                .is_some_and(|d| d.source() == gdk::InputSource::Touchscreen)
            {
                2
            } else {
                0
            };
            let result = u
                .model
                .borrow_mut()
                .down(x, y, pressure, g.current_event_time(), device);
            if let Err(e) = result {
                u.error(&format!("Input failed: {e:?}"));
                u.model.borrow_mut().cancel();
            }
            u.refresh();
        });
    }
    {
        let u = ui.clone();
        stylus.connect_motion(move |g, x, y| {
            let result = (|| {
                let mut m = u.model.borrow_mut();
                if matches!(m.gesture, Gesture::Ink) {
                    // Backlog axes share GDK coordinates; translate to this widget like current x/y.
                    let offset = (
                        x - g.axis(gdk::AxisUse::X).unwrap_or(x),
                        y - g.axis(gdk::AxisUse::Y).unwrap_or(y),
                    );
                    for event in g.backlog().unwrap_or_default() {
                        if event
                            .flags()
                            .contains(gdk::AxisFlags::X | gdk::AxisFlags::Y)
                        {
                            let axes = event.axes();
                            let pressure = event
                                .flags()
                                .contains(gdk::AxisFlags::PRESSURE)
                                .then_some(axes[gdk::AxisUse::Pressure.into_glib() as usize]);
                            m.motion(
                                axes[gdk::AxisUse::X.into_glib() as usize] + offset.0,
                                axes[gdk::AxisUse::Y.into_glib() as usize] + offset.1,
                                pressure,
                                event.time(),
                            )?;
                        }
                    }
                }
                m.motion(x, y, g.axis(gdk::AxisUse::Pressure), g.current_event_time())
            })();
            if result.is_err() {
                u.model.borrow_mut().cancel();
            }
            u.refresh();
        });
    }
    {
        let u = ui.clone();
        stylus.connect_up(move |g, x, y| {
            let result = {
                let mut m = u.model.borrow_mut();
                m.motion(x, y, g.axis(gdk::AxisUse::Pressure), g.current_event_time())
                    .and_then(|_| m.up())
            };
            if result.is_err() {
                u.model.borrow_mut().cancel();
            }
            u.refresh();
        });
    }
    {
        let u = ui.clone();
        stylus.connect_cancel(move |_, _| {
            u.model.borrow_mut().cancel();
            u.refresh();
        });
    }
    canvas.add_controller(stylus);
}
