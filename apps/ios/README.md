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

## Apple Watch copies

On iPhone, Document → Send Copy to Apple Watch takes an explicit Rust document
snapshot after cancelling temporary input and queues a `.nsketch` copy through
WatchConnectivity. The sender rejects copies larger than 8 MiB, explains pairing
or installation failures, and retains the outgoing file until the transfer
completion callback. A queued transfer may arrive later; transport completion does
not certify import on Watch. Sending never marks the editing document saved.

The Watch app must be embedded in the iPhone package and provisioned for a real
paired device; `build-ios` alone intentionally remains independent. No automatic
sync or Watch editing is provided. Actual paired hardware delivery is unverified.
Simulator-only adapter replay also checks oversized-copy rejection without sending.

Document → Apple Watch Transfer Status retains the latest transfer message for the
current session, including a failure received while another sheet is open. The
queue confirmation does not claim that a previously saved document became dirty.

## Adapter organization

`MobileEditor` retains the legacy Rust ABI 2 ownership boundary. `DrawingController`
composes the native window content and keeps responder identity; its named extensions
separate interface setup, editing actions, native inspector dialogs, document picker
callbacks, viewport operations, Watch transfer presentation and debug replay.
`DrawingView` owns viewport and contact lifetime, with separate touch, spatial-input,
stroke-input and Core Graphics painting implementations. Touch overrides forward to
the same arbitration code; rotation and second-contact cancellation order is unchanged.
`NativeSketch.swift` now contains only application and scene lifecycle composition.

This is a behavior-preserving extraction, not portable application-command or Skia
adoption. Canvas/tool state and synchronous file policy still need the task 022 typed
facade migration. Native field priority, UIKit selectors and existing controls remain.
The shared legacy module map lives in `apps/apple-bridge/CSketchFFI`; macOS does not use it.

Use `node scripts/build.ts --platform ios --variant simulator` for a background build;
`--platform visionos` compiles the same editor adapters for the 2D visionOS window.
Building does not constitute native touch, Pencil, spatial input or accessibility acceptance.
