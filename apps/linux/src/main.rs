use adw::prelude::*;
mod activation;
#[cfg(test)]
mod adapter_tests;
mod canvas_painter;
mod command_owner;
mod composition;
mod control_actions;
mod controls;
mod diagnostic_budget;
#[cfg(test)]
mod diagnostic_receipt_tests;
mod diagnostic_sink;
#[cfg(test)]
mod diagnostic_sink_tests;
mod diagnostics;
mod file_dialogs;
mod file_effects;
mod keyboard_input;
#[cfg(test)]
mod load_order_tests;
mod model;
mod navigation_input;
#[cfg(test)]
mod ordering_tests;
mod pointer_translation;
mod presenter;
mod provider;
#[cfg(test)]
mod provider_tests;
mod provider_work;
mod runtime;
#[cfg(test)]
mod save_policy_tests;
mod scene_painter;
#[cfg(test)]
mod scene_tests;
mod selection_painter;
mod shortcut_intent;
mod stylus_input;
#[cfg(test)]
mod test_support;

fn main() {
    crate::diagnostics::start();
    let app = adw::Application::builder()
        .application_id("dev.nativesketch.Gtk")
        .build();
    app.connect_activate(activation::activate);
    app.run();
}
