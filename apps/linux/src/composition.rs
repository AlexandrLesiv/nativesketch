use crate::{controls::Controls, model::Model, presenter::Ui};
use adw::prelude::*;
use std::{cell::RefCell, rc::Rc};

pub(crate) fn build(app: &adw::Application) {
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
    let controls = Controls::new(&header, &root);
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
        undo: controls.undo.clone(),
        redo: controls.redo.clone(),
        selected: vec![
            controls.scale.clone().upcast(),
            controls.apply.clone().upcast(),
            controls.delete.clone().upcast(),
        ],
        cycling: vec![
            controls.previous.clone().upcast(),
            controls.next.clone().upcast(),
        ],
    };
    crate::control_actions::connect(&ui, &controls);
    crate::canvas_painter::connect(&ui);
    crate::stylus_input::connect(&ui);
    crate::navigation_input::connect(&ui);
    crate::keyboard_input::connect(&ui, &controls);
    ui.refresh();
    window.present();
}
