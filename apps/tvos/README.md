# Apple TV viewer

UIKit with focusable native remote controls. Open an explicit HTTPS `.nsketch` URL
(up to 64 MiB) or the bundled Rust-generated Sample. Fit, Zoom and directional Pan
change only the viewport. There is no editing, save, undo, or selection interface.
Loading can be cancelled; a new request invalidates old callbacks. Download bodies
are bounded while streaming and redirects cannot downgrade HTTPS. Rust validates a
candidate before replacing the last good copy. A local last-viewed cache is best
effort and may be purged by tvOS; this is not a durable document library.

Run `scripts/build-tvos simulator` (or `device`), then
`scripts/run-tvos <Apple-TV-simulator-UUID> [--integration-test]`.
Requires Xcode and official Rust targets `aarch64-apple-tvos[-sim]`. tvOS 17+,
Apple silicon build host. Device builds require owner provisioning before installation.
The provisional Core Graphics painter does not decide the production renderer.

`scripts/check-apple-viewers` runs the real Rust-backed adapter regressions on macOS;
that is separate from simulator launch, remote focus/VoiceOver and physical Apple TV tests.
