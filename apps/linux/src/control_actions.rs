use crate::{controls::Controls, presenter::Ui};
use adw::prelude::*;
use gtk::glib;
use sketch_app::{AppError, Intent};
use sketch_core::application::{routing::*, *};
pub(crate) fn connect(ui: &Ui, controls: &Controls) {
    for (button, action) in [
        (&controls.undo, EditorAction::Undo),
        (&controls.redo, EditorAction::Redo),
        (
            &controls.previous,
            EditorAction::CycleSelection(CycleDirection::Previous),
        ),
        (
            &controls.next,
            EditorAction::CycleSelection(CycleDirection::Next),
        ),
        (&controls.delete, EditorAction::DeleteSelection),
    ] {
        let ui = ui.clone();
        button.connect_clicked(move |_| {
            ui.action(|model| model.owner.dispatch(Intent::Action(action)))
        });
    }
    for (button, factor) in [
        (&controls.out, Some(0.8)),
        (&controls.reset, None),
        (&controls.zoom_in, Some(1.25)),
    ] {
        let ui = ui.clone();
        button.connect_clicked(move |_| {
            ui.action(|model| {
                model.zoom_to(factor.map_or(1.0, |factor| model.zoom * factor));
                Ok(())
            })
        });
    }
    let ui_scale = ui.clone();
    let scale = controls.scale.clone();
    controls.apply.connect_clicked(move |_| {
        ui_scale.action(|model| {
            let factor =
                ScaleFactor::try_from(scale.value() / 100.0).map_err(|_| AppError::Invalid)?;
            model
                .owner
                .dispatch(Intent::Action(EditorAction::ScaleSelection {
                    factor,
                    anchor: ScaleAnchor::SelectionCenter,
                }))
        })
    });
    for (button, tool) in [
        (&controls.pen, ToolId::Pen),
        (&controls.select, ToolId::Select),
    ] {
        let ui = ui.clone();
        button.connect_toggled(move |button| {
            if button.is_active()
                && ui
                    .model
                    .borrow()
                    .owner
                    .app
                    .projection(ui.model.borrow().owner.session)
                    .is_ok_and(|p| p.editor.tool != tool)
            {
                ui.action(|model| {
                    model
                        .owner
                        .dispatch(Intent::Action(EditorAction::ChooseTool(tool)))
                });
            }
        });
    }
    let ui_color = ui.clone();
    controls.color.connect_rgba_notify(move |color| {
        ui_color.action(|model| {
            let mut brush = model
                .owner
                .app
                .projection(model.owner.session)?
                .editor
                .brush
                .brush();
            let color = color.rgba();
            brush.rgba = [
                color.red() as f64,
                color.green() as f64,
                color.blue() as f64,
                color.alpha() as f64,
            ];
            model.owner.dispatch(Intent::Edit(EditorCommand::SetBrush(
                brush.try_into().map_err(|_| AppError::Invalid)?,
            )))
        })
    });
    let ui_width = ui.clone();
    controls.width.connect_value_changed(move |width| {
        ui_width.action(|model| {
            let mut brush = model
                .owner
                .app
                .projection(model.owner.session)?
                .editor
                .brush
                .brush();
            brush.width = width.value();
            model.owner.dispatch(Intent::Edit(EditorCommand::SetBrush(
                brush.try_into().map_err(|_| AppError::Invalid)?,
            )))
        })
    });
    for (button, intent) in [
        (&controls.save, FileIntent::SaveAs),
        (&controls.open, FileIntent::Navigate(NavigationIntent::Open)),
        (&controls.new, FileIntent::Navigate(NavigationIntent::New)),
    ] {
        let ui = ui.clone();
        button.connect_clicked(move |_| ui.file(intent));
    }
    let close = ui.clone();
    ui.window.connect_close_request(move |_| {
        close.file(FileIntent::Navigate(NavigationIntent::Close));
        glib::Propagation::Stop
    });
    let inactive = ui.clone();
    ui.window.connect_is_active_notify(move |window| {
        if !window.is_active() {
            inactive.model.borrow_mut().cancel();
            inactive.refresh();
        }
    });
}
