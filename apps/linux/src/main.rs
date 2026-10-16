use adw::prelude::*;
#[cfg(test)]
mod adapter_tests;
mod canvas_painter;
mod composition;
mod control_actions;
mod controls;
mod file_effects;
mod keyboard_input;
mod model;
mod navigation_input;
mod presenter;
mod stylus_input;

fn main() {
    let app = adw::Application::builder()
        .application_id("dev.nativesketch.Gtk")
        .build();
    app.connect_activate(composition::build);
    app.run();
}
