use crate::presenter::Ui;
use adw::prelude::*;
use gtk::glib;
use sketch_core::application::routing::EffectDecision;
use std::time::{Duration, Instant};
pub(crate) fn connect(ui: &Ui, app: &adw::Application) {
    let hold = app.hold();
    let ui = ui.clone();
    let mut last_warning = Instant::now();
    let mut warnings = 0;
    let mut closing_since = None;
    glib::timeout_add_local(Duration::from_millis(10), move || {
        let before = {
            let model = ui.model.borrow();
            model.owner.app.projection(model.owner.session)
        };
        let result = {
            let mut model = ui.model.borrow_mut();
            ui.provider.borrow_mut().poll(&mut model.owner)
        };
        if result.is_err() && warnings < 4 && last_warning.elapsed() >= Duration::from_secs(30) {
            crate::diagnostics::record(format_args!("event=provider_stalled error={result:?}"));
            warnings += 1;
            last_warning = Instant::now();
        }
        if ui.closing.get() {
            let since = closing_since.get_or_insert_with(Instant::now);
            if since.elapsed() >= Duration::from_secs(30)
                && warnings < 4
                && last_warning.elapsed() >= Duration::from_secs(30)
            {
                crate::diagnostics::record(format_args!(
                    "event=shutdown_waiting_for_actual_provider_termination"
                ));
                warnings += 1;
                last_warning = Instant::now();
            }
            if ui
                .provider
                .borrow_mut()
                .stop_when_drained(&ui.model.borrow().owner)
            {
                ui.window.destroy();
                crate::activation::retired();
                return glib::ControlFlow::Break;
            }
        } else {
            if let Some(token) = ui.awaiting.get() {
                let ready = ui
                    .model
                    .borrow()
                    .owner
                    .app
                    .projection(ui.model.borrow().owner.session)
                    .is_ok_and(|projection| projection.outstanding_saves == 0);
                if ready {
                    ui.awaiting.set(None);
                    if let Ok(context) = ui.effect_context(token) {
                        ui.reply(context, token, EffectDecision::Continue);
                    }
                }
            }
            ui.process_effects();
            // Refresh after provider completion only; no permanent frame loop.
            let changed = {
                let model = ui.model.borrow();
                model.owner.app.projection(model.owner.session) != before
            };
            if changed {
                ui.refresh();
            }
        }
        let _keep_alive = &hold;
        glib::ControlFlow::Continue
    });
}
