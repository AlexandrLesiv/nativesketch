# UIKit iPhone / iPad host

Swift/UIKit native navigation, toolbars, menus, color picker and Files picker call the
shared Rust ABI 2. UIKit's current SDK supplies system chrome; no custom imitation of
Liquid Glass is painted over the canvas. CoreGraphics paints Rust paths provisionally.

Install Xcode with iOS SDKs and Rust targets `aarch64-apple-ios-sim` and
`aarch64-apple-ios`. `scripts/build-ios simulator` and `scripts/build-ios device`
produce apps under `artifacts/ios-simulator/` and `artifacts/ios-device/`.
`DEVELOPER_DIR` defaults per command to `/Applications/Xcode.app/Contents/Developer`;
the scripts do not change the machine's selected developer directory.

```sh
./scripts/build-ios simulator
./scripts/run-ios <booted-simulator-UUID>
./scripts/run-ios <booted-simulator-UUID> --integration-test
```

Pen/Select remains visible in a 150pt segmented control with a 44pt height. The native
Drawing Options menu exposes Style, Selection and Zoom without clipping a narrow
phone toolbar. One contact draws/selects; a second cancels the draft before navigation.
Coalesced actual touches and available Pencil force reach Rust. Rotation cancels input
and preserves the viewport center. Keyboard commands preserve text editing.

Files import streams at most 64 MiB plus one sentinel byte before Rust validates the
entire document. Save to Files exports a temporary snapshot and only acknowledges
its revision after a successful picker result. New/Open provide Save/Discard/Cancel.
Cancellation or invalid input retains the current document. I/O remains synchronous.

Device output is a development artifact; installing on a physical device requires
owner provisioning/signing. Simulator replays are synthetic, not physical touch or
Pencil evidence. See the central evidence report for the tested revision and actual
runtime coverage. No autosave/process-termination recovery, multiwindow editing,
physical Pencil certification, VoiceOver speech or release distribution is provided.
