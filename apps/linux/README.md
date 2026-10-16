# GTK native host

`./scripts/build-linux` builds the standalone GTK4/libadwaita application and copies it
to `artifacts/linux/native-sketch-gtk`. Run that executable on the build host.
`./scripts/check-linux` checks formatting, clippy, an adapter replay, and the build.
Install Rust, pkg-config and development packages for GTK >=4.10 and libadwaita >=1.5
(for example distro `libgtk-4-dev` and `libadwaita-1-dev`). Dependencies are pinned by
this standalone crate's Cargo.lock. The shared Rust core is a repository path dependency.

Actual GTK/libadwaita widgets provide the header bar, grouped tool buttons, color/number
controls and file dialogs. The horizontal controls can scroll on a narrow window.
The white drawing surface uses a provisional Cairo nonzero path painter fed exclusively
by sketch-core. Skia is selected; this host still awaits renderer migration.

Pen draws; Select hit-tests and exposes corner resize handles. Previous/Next stroke and
numeric scale provide accessible editing alternatives. Space-drag/scroll pan; Control
scroll and visible zoom buttons zoom. Two-contact GTK gestures cancel drafts before
navigation. Escape, focus loss, capture cancellation and layout changes cancel temporary
input. Canvas shortcuts P/V/Delete, Control/Command-Z, S/O/N do not steal text entry.
Save uses the Rust atomic path writer; failed writes remain dirty. Open uses the Rust
bounded validator and preserves the document on failure. New/Open/Close offer native
Save/Discard/Cancel. Non-local file-provider locations are rejected with an explicit
message; no unsafe remote-file overwrite fallback is used.

Verification so far: compiled and adapter-tested on macOS with Homebrew GTK4 4.24.1 and
libadwaita 1.10.0. The GTK window and tool controls also ran on Mac, but synthetic
mouse movement omitted the held-button flag and correctly cancelled ink; real mouse
drawing remains unverified. See the platform evidence report. This does **not** verify Linux Wayland/X11, native file portals, desktop
packaging, physical tablet pressure, accessibility speech, or pen latency. Actual Linux
runtime testing remains required. Unsaved process-crash recovery is not implemented.

## Native adapter composition

`composition.rs` creates the window and connects collaborators. `controls.rs` owns native
widget construction and `control_actions.rs` retains existing signal bindings.
`presenter.rs` projects editor state into controls, status, accessibility labels and errors;
`file_effects.rs` owns native chooser/unsaved dialogs. `model.rs` retains the existing Rust
editor plus temporary GTK capture/viewport state. Pointer, keyboard/focus and navigation
callbacks are isolated in `stylus_input.rs`, `keyboard_input.rs` and `navigation_input.rs`.
`canvas_painter.rs` retains the existing Cairo path presentation. Each handwritten module
is at most 200 physical lines after rustfmt.

This extraction preserves existing widgets, strings, callback ordering and editor behavior.
It does not complete the portable application migration: command/save/navigation policy
still uses the legacy Editor API, and Cairo remains until the task 022 Skia consumer slice.
Skia is selected; renderer selection is not reopened by retaining this interim painter.
Existing Cairo result handling and input-error diagnostics still need the shared diagnostics
migration rather than being described as fully hardened here.

`adapter_tests.rs::adapter_cancel_transform_roundtrip` preserves the existing background
replay for stroke commit/cancel, timestamps, transform undo, corrupt import, centered resize
and failed-save dirty state (INK-001, INPUT-001, CANCEL-001, UNDO-001, LOAD-001, SAVE-001).
Run `cargo fmt --manifest-path apps/linux/Cargo.toml -- --check`,
`cargo clippy --locked --manifest-path apps/linux/Cargo.toml -- -D warnings`, and
`cargo test --locked --manifest-path apps/linux/Cargo.toml` from the repository root.
These checks do not initialize GTK or present windows. They do not prove native input,
Wayland/X11, Linux portals, accessibility, physical stylus or compositor behavior.
