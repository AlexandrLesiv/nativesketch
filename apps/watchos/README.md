# Apple Watch viewer

SwiftUI preview with Digital Crown zoom, drag pan and Fit. Always View Only; no
editing, saving, selection, or undo controls. Empty state asks to send a copy from
iPhone. The iPhone companion is `dev.nativesketch.ios`; the watch app identifier is
`dev.nativesketch.ios.watch`. This is a companion app, not a standalone cloud service.

WatchConnectivity `transferFile` carries the complete `.nsketch` document and metadata:
`protocolVersion: Int = 1`, `sentAt: Double` (finite positive Unix seconds),
`title: String` (1–512 characters), and `contentType: "dev.nativesketch.document"`.
The receiver moves the temporary file before the delegate returns, reads at most
8 MiB, and validates with the shared Rust engine. Older/equal timestamps and invalid
copies preserve the last good document. Metadata and bytes share one atomic local
envelope; the UI shows the local receipt time. Sending a copy does not save or alter
the phone document. Transfers require an actually paired phone/watch; simulator
adapter replay does not prove WatchConnectivity delivery.

Run `scripts/build-watchos simulator` (or `device`), then
`scripts/run-watchos <Watch-simulator-UUID> [--integration-test]`.
Packaging the companion under an iOS app's `Watch` directory is handled by the
Apple family build. Xcode and official Rust `aarch64-apple-watchos[-sim]` required.
Minimum watchOS 10, **arm64 watches only**; older arm64_32 watches are not supported
by this build. Real device installation requires owner provisioning.
The Core Graphics/SwiftUI painter is provisional; production renderer remains open.

`scripts/check-apple-viewers` covers real Rust import/geometry, invalid/oversized
input preservation, stale Watch copies, bounded file reads and atomic cache restore.
Crown, accessibility, paired delivery, memory and physical-device checks are separate.

Simulator builds alone define `DEBUG`: `--integration-test` exits with status 0/1;
`--preview-sample` loads a deliberately long-titled synthetic copy through the real
receiver store for visual inspection. This fixture is explicitly labelled as a
simulator sample and is not paired transfer evidence. Device builds exclude both
entry points and the replay implementation. Details opens a scrollable full title,
receipt date and status; the compact preview keeps these as separate elements.
