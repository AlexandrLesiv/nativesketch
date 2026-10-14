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
by sketch-core. No permanent production renderer selection is implied.

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
