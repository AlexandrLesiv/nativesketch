use adw::prelude::*;
use glib::translate::IntoGlib;
use gtk::{gdk, gio, glib};
use sketch_core::{Brush, Editor, Input, Sample};
use std::{cell::RefCell, rc::Rc};

struct Model {
    editor: Editor,
    zoom: f64,
    offset: (f64, f64),
    size: (i32, i32),
    select: bool,
    space: bool,
    brush: Brush,
    sample: u64,
    start_time: u32,
    gesture: Gesture,
}
#[derive(Default)]
enum Gesture {
    #[default]
    Idle,
    Ink,
    Pan(f64, f64),
    Resize(f64, f64, f64),
}
impl Model {
    fn new() -> Self {
        Self {
            editor: Editor::new(),
            zoom: 1.0,
            offset: (0.0, 0.0),
            size: (0, 0),
            select: false,
            space: false,
            brush: Brush {
                rgba: [0.14, 0.39, 0.92, 1.0],
                width: 4.0,
            },
            sample: 0,
            start_time: 0,
            gesture: Gesture::Idle,
        }
    }
    fn cancel(&mut self) {
        self.editor.cancel();
        self.gesture = Gesture::Idle;
    }
    fn point(&self, x: f64, y: f64) -> (f64, f64) {
        (
            (x - self.offset.0) / self.zoom,
            (y - self.offset.1) / self.zoom,
        )
    }
    fn zoom_to(&mut self, value: f64) {
        self.cancel();
        let (x, y) = (self.size.0 as f64 / 2.0, self.size.1 as f64 / 2.0);
        let p = self.point(x, y);
        self.zoom = value.clamp(0.05, 32.0);
        self.offset = (x - p.0 * self.zoom, y - p.1 * self.zoom);
    }
    fn layout(&mut self, w: i32, h: i32) {
        if self.size != (w, h) {
            self.cancel();
            if self.size.0 != 0 {
                self.offset.0 += (w - self.size.0) as f64 / 2.0;
                self.offset.1 += (h - self.size.1) as f64 / 2.0;
            }
            self.size = (w, h);
        }
    }
    fn down(
        &mut self,
        x: f64,
        y: f64,
        pressure: Option<f64>,
        time: u32,
        device: u32,
    ) -> sketch_core::Result<()> {
        self.cancel();
        let p = self.point(x, y);
        if self.space {
            self.gesture = Gesture::Pan(x, y);
            return Ok(());
        }
        if self.select {
            let s = self.editor.state();
            let b = s.bounds;
            if s.selected != 0 {
                let corners = [
                    (b.x, b.y),
                    (b.x + b.width, b.y),
                    (b.x, b.y + b.height),
                    (b.x + b.width, b.y + b.height),
                ];
                if let Some(i) = corners
                    .iter()
                    .position(|c| (p.0 - c.0).hypot(p.1 - c.1) < 14.0 / self.zoom)
                {
                    let a = corners[3 - i];
                    self.editor.resize_begin()?;
                    self.gesture =
                        Gesture::Resize(a.0, a.1, (p.0 - a.0).hypot(p.1 - a.1).max(0.01));
                    return Ok(());
                }
            }
            self.editor.select_at(p.0, p.1, 8.0 / self.zoom)?;
        } else {
            self.editor.begin(1, device, self.brush)?;
            self.sample = 0;
            self.start_time = time;
            self.gesture = Gesture::Ink;
            self.motion(x, y, pressure, time)?;
        }
        Ok(())
    }
    fn motion(
        &mut self,
        x: f64,
        y: f64,
        pressure: Option<f64>,
        time: u32,
    ) -> sketch_core::Result<()> {
        let p = self.point(x, y);
        match self.gesture {
            Gesture::Ink => {
                self.sample += 1;
                self.editor.append(
                    1,
                    &[Input::Actual(Sample {
                        x: p.0,
                        y: p.1,
                        timestamp: time.wrapping_sub(self.start_time) as f64 / 1000.0,
                        pressure,
                        id: self.sample,
                    })],
                )?;
            }
            Gesture::Pan(a, b) => {
                self.offset.0 += x - a;
                self.offset.1 += y - b;
                self.gesture = Gesture::Pan(x, y);
            }
            Gesture::Resize(a, b, d) => {
                self.editor
                    .resize_update(((p.0 - a).hypot(p.1 - b) / d).max(0.01), a, b)?
            }
            Gesture::Idle => {}
        }
        Ok(())
    }
    fn up(&mut self) -> sketch_core::Result<()> {
        match self.gesture {
            Gesture::Ink => self.editor.commit(1)?,
            Gesture::Resize(..) => self.editor.resize_commit()?,
            _ => {}
        }
        self.gesture = Gesture::Idle;
        Ok(())
    }
}
type Shared = Rc<RefCell<Model>>;
#[derive(Clone)]
struct Ui {
    model: Shared,
    window: adw::ApplicationWindow,
    canvas: gtk::DrawingArea,
    status: gtk::Label,
    undo: gtk::Button,
    redo: gtk::Button,
    selected: Vec<gtk::Widget>,
    cycling: Vec<gtk::Widget>,
}
impl Ui {
    fn refresh(&self) {
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
    fn action(&self, f: impl FnOnce(&mut Model) -> sketch_core::Result<()>) {
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
    fn error(&self, text: &str) {
        let dialog = adw::AlertDialog::builder()
            .heading("Could not complete action")
            .body(text)
            .build();
        dialog.add_response("ok", "OK");
        dialog.present(Some(&self.window));
    }
    fn save(&self, then: Option<Rc<dyn Fn()>>) {
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
                if let Ok(file) = result {
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
    fn guard(&self, then: Rc<dyn Fn()>) {
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
    fn open(&self) {
        let ui = self.clone();
        self.guard(Rc::new(move || {
            let dialog = gtk::FileDialog::builder().title("Open Drawing").build();
            let inner = ui.clone();
            dialog.open(Some(&ui.window), None::<&gio::Cancellable>, move |result| {
                if let Ok(file) = result {
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
fn button(label: &str) -> gtk::Button {
    gtk::Button::with_label(label)
}
fn main() {
    let app = adw::Application::builder()
        .application_id("dev.nativesketch.Gtk")
        .build();
    app.connect_activate(build);
    app.run();
}
fn build(app: &adw::Application) {
    let model = Rc::new(RefCell::new(Model::new()));
    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title("Native Sketch")
        .default_width(1100)
        .default_height(760)
        .build();
    let root = gtk::Box::new(gtk::Orientation::Vertical, 0);
    let header = adw::HeaderBar::new();
    root.append(&header);
    let new = button("New");
    let open = button("Open…");
    let save = button("Save…");
    let undo = button("Undo");
    let redo = button("Redo");
    header.pack_start(&new);
    header.pack_start(&open);
    header.pack_start(&save);
    header.pack_end(&redo);
    header.pack_end(&undo);
    let bar = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    bar.set_margin_start(12);
    bar.set_margin_end(12);
    bar.set_margin_top(8);
    bar.set_margin_bottom(8);
    let scroll = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Automatic)
        .vscrollbar_policy(gtk::PolicyType::Never)
        .child(&bar)
        .build();
    root.append(&scroll);
    let pen = gtk::ToggleButton::with_label("Pen");
    pen.set_active(true);
    let select = gtk::ToggleButton::with_label("Select");
    select.set_group(Some(&pen));
    bar.append(&pen);
    bar.append(&select);
    let color = gtk::ColorDialogButton::new(Some(gtk::ColorDialog::new()));
    color.set_rgba(&gdk::RGBA::new(0.14, 0.39, 0.92, 1.0));
    color.set_tooltip_text(Some("Pen color"));
    color.update_property(&[gtk::accessible::Property::Label("Pen color")]);
    bar.append(&color);
    bar.append(&gtk::Label::new(Some("Width")));
    let width = gtk::SpinButton::with_range(0.1, 100.0, 0.5);
    width.set_value(4.0);
    width.update_property(&[gtk::accessible::Property::Label("Pen width")]);
    bar.append(&width);
    let previous = button("Previous stroke");
    let next = button("Next stroke");
    bar.append(&previous);
    bar.append(&next);
    bar.append(&gtk::Label::new(Some("Scale %")));
    let scale = gtk::SpinButton::with_range(1.0, 10000.0, 10.0);
    scale.set_value(100.0);
    scale.update_property(&[gtk::accessible::Property::Label("Selection scale percent")]);
    bar.append(&scale);
    let apply = button("Apply");
    bar.append(&apply);
    let delete = button("Delete");
    bar.append(&delete);
    let out = button("−");
    out.set_tooltip_text(Some("Zoom out"));
    let reset = button("100%");
    let zoom_in = button("+");
    zoom_in.set_tooltip_text(Some("Zoom in"));
    bar.append(&out);
    bar.append(&reset);
    bar.append(&zoom_in);
    let canvas = gtk::DrawingArea::builder()
        .hexpand(true)
        .vexpand(true)
        .focusable(true)
        .build();
    root.append(&canvas);
    let status = gtk::Label::new(None);
    status.set_margin_top(6);
    status.set_margin_bottom(6);
    root.append(&status);
    window.set_content(Some(&root));
    let ui = Ui {
        model: model.clone(),
        window: window.clone(),
        canvas: canvas.clone(),
        status,
        undo: undo.clone(),
        redo: redo.clone(),
        selected: vec![
            scale.clone().upcast(),
            apply.clone().upcast(),
            delete.clone().upcast(),
        ],
        cycling: vec![previous.clone().upcast(), next.clone().upcast()],
    };
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
                    Ok(())
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
    {
        let m = model.clone();
        canvas.set_draw_func(move |_, cr, w, h| {
            let mut m = m.borrow_mut();
            m.layout(w, h);
            cr.set_source_rgb(1.0, 1.0, 1.0);
            let _ = cr.paint();
            cr.translate(m.offset.0, m.offset.1);
            cr.scale(m.zoom, m.zoom);
            let mut id = None;
            for command in m.editor.render() {
                if id != Some(command.stroke_id) {
                    if id.is_some() {
                        let _ = cr.fill();
                    }
                    cr.new_path();
                    id = Some(command.stroke_id);
                    let c = command.rgba;
                    cr.set_source_rgba(c[0], c[1], c[2], c[3]);
                }
                let p = command.points;
                match command.verb {
                    0 => cr.move_to(p[0], p[1]),
                    1 => cr.line_to(p[0], p[1]),
                    2 => cr.curve_to(p[0], p[1], p[2], p[3], p[4], p[5]),
                    3 => cr.close_path(),
                    _ => {}
                }
            }
            let _ = cr.fill();
            let s = m.editor.state();
            if s.selected != 0 {
                let b = s.bounds;
                cr.set_source_rgb(0.05, 0.35, 0.85);
                cr.set_line_width(1.0 / m.zoom);
                cr.rectangle(b.x, b.y, b.width, b.height);
                let _ = cr.stroke();
                for (x, y) in [
                    (b.x, b.y),
                    (b.x + b.width, b.y),
                    (b.x, b.y + b.height),
                    (b.x + b.width, b.y + b.height),
                ] {
                    cr.rectangle(
                        x - 4.0 / m.zoom,
                        y - 4.0 / m.zoom,
                        8.0 / m.zoom,
                        8.0 / m.zoom,
                    );
                    cr.set_source_rgb(1.0, 1.0, 1.0);
                    let _ = cr.fill_preserve();
                    cr.set_source_rgb(0.05, 0.35, 0.85);
                    let _ = cr.stroke();
                }
            }
        });
    }
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
            } else if ctrl && key == gdk::Key::z {
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
    ui.refresh();
    window.present();
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn adapter_cancel_transform_roundtrip() {
        let mut m = Model::new();
        m.layout(500, 400);
        m.down(40.0, 50.0, None, 100, 0).unwrap();
        m.motion(80.0, 80.0, None, 127).unwrap();
        m.up().unwrap();
        assert_eq!(m.editor.state().count, 1);
        assert_eq!(m.editor.document().strokes[0].samples[1].timestamp, 0.027);
        m.down(90.0, 100.0, None, 200, 0).unwrap();
        m.cancel();
        assert_eq!(m.editor.state().count, 1);
        m.editor.select_cycle(1).unwrap();
        m.editor.resize_begin().unwrap();
        m.editor.resize_update(2.0, 0.0, 0.0).unwrap();
        m.editor.resize_commit().unwrap();
        m.editor.undo().unwrap();
        let before = m.editor.export().unwrap();
        assert!(m.editor.import(b"bad").is_err());
        assert_eq!(before, m.editor.export().unwrap());
        let center = m.point(250.0, 200.0);
        m.layout(700, 600);
        assert_eq!(center, m.point(350.0, 300.0));
        let path = std::env::temp_dir().join(format!("native-sketch-gtk-{}", std::process::id()));
        std::fs::create_dir_all(&path).unwrap();
        assert!(m.editor.save(&path).is_err());
        assert!(m.editor.state().dirty);
        std::fs::remove_dir(path).unwrap();
    }
}
