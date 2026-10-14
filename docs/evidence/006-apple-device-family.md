# Apple device family evidence

Branch `codex/apple-device-family`, based on `30d472b`. Main remains `b63a223`.
Native first-stroke prototypes, not signed or distributed releases. Production renderer
selection remains open; existing platform path painters consume Rust geometry.

## Capability and implementation

Mac/AppKit and iPhone/iPad/UIKit retain editing. Vision Pro adds a native UIKit 2D editing
window with explicit Pen, Select/Pan, indirect navigation and native Files access.
Apple TV and Apple Watch are view only: a private Swift adapter imports into Rust and
reads paths/state, with no editing API exposed by the adapter or editing controls.

TV accepts explicit HTTPS document URLs up to 64 MiB, validates candidates before
replacement and uses a last-viewed cache that the OS may purge. Watch receives explicit
iPhone copies up to 8 MiB through WatchConnectivity. Validated bytes and metadata share
an atomic cache; invalid or stale copies retain the last good document. Sending never
acknowledges a save on the phone. Copies are not live synchronization.

## Tested revisions

- `b2c7133`: required `scripts/check` passed (board, Rust formatting/clippy, 17 Rust
  tests, macOS build and Swift/Rust integration). iPhone and Vision simulator replays
  passed including large-document Fit/navigation, unchanged bytes and retained Watch
  errors during a modal. Equivalent worker `81f7628` built iOS and visionOS device/sim.
- `25e67f2`: fresh reviewer independently reproduced download-policy boundaries,
  HTTPS downgrade rejection and retained HTTP/oversize causes; no actionable findings
  after the session-access race repair. Shared viewer replay independently passed
  Rust byte identity, corrupt/oversized/stale retention, bounded reads and cache failures.
- `a092b53`: viewer debug-only fixture/replay entrypoints. Complete simulator family
  build and embedded Watch packaging passed; shared host and Watch runtime replays passed.
- `070412b`: final code revision. Complete device family build passed. Final Watch
  simulator build passed and its embedded iPhone copy was refreshed. TV runtime replay
  passed (TV sources unchanged since `a092b53`). Watch small-screen and accessibility
  text repairs were inspected live; fresh reviewer reported no actionable findings.
  Equivalent viewer worker `3e3adcd` built both simulator and device variants.

Generated logs and screenshots are under `artifacts/apple-family-verification/`.
The tests use disposable local data. Native UI and simulator execution were coordinated
exclusively by the lead; concurrent implementation used isolated worktrees.

## Review and repairs

The configured designer defined device capabilities before implementation and reviewed
source and visible layouts. A fresh reviewer independently reviewed the implementation.
Repairs include Fit below the ordinary zoom floor, no upward zoom jump after fitting,
accurate saved-status messages, retained asynchronous Watch errors, synchronized TV
session lifetime, specific HTTP/size errors, separate Watch name/date/status, and disabled
unavailable TV commands. Runtime inspection found and repaired a collapsed Watch canvas.
The designer accepted the corrected 40 mm layout at normal text size and reachable
controls at accessibility text size. The Details button visually truncates at the largest
inspected size but retains its full accessibility name and opens the full details sheet;
this remains a minor polish limitation. Test-only fixtures are excluded from device builds.

## Build and runtime coverage

Host: Apple M2 Pro, 16 GiB, macOS 27.0.1; Xcode/SDKs and simulator runtimes 27,
Swift 6.4, Rust 1.99. All builds here target ARM64. Device builds are compilation and
local ad-hoc packaging evidence, not provisioned physical installation.

| Host | Build/check evidence | Visible/runtime evidence |
| --- | --- | --- |
| macOS | Required integration check and final native build passed | Existing AppKit editor; unchanged in this task |
| iPhone/iPad | Simulator/device builds passed; iPhone adapter replay passed | iPhone native Watch-copy menu and unavailable-companion error inspected; iPad-specific runtime not rerun in this task |
| Vision Pro | Simulator/device builds and adapter replay passed, including large-document Fit and initial Select | Native 2D window inspected; spatial pointer drawing not verified |
| Apple TV | Simulator/device builds and read-only replay passed | Native remote focus, Sample, Zoom and Fit exercised; HTTPS policy tested in harness, live URL download not exercised |
| Apple Watch | Simulator/device builds and read-only replay passed | 40 mm sample preview, normal/large text, scrolling, Fit/Details layout and native Details sheet inspected |

The paired simulators reported the Watch companion unavailable despite the nested
companion bundle being installed. No successful phone-to-Watch delivery is claimed.
Apple documents that [file transfers are unsupported in Simulator](https://developer.apple.com/documentation/watchconnectivity/wcsession/transferfile(_:metadata:));
end-to-end delivery requires provisioned paired devices. Import/cache behavior uses
actual Rust in the automated replay, and the visual fixture is explicitly synthetic.

Logs: `integration-check.txt`, `family-build.txt`, `family-device-build.txt`,
`viewer-check.txt`, `iphone-replay.txt`, `vision-replay.txt`, `tv-replay.txt`,
`watch-replay.txt` and `watch-build.txt`. Screenshots `watch.png` and `watch-large.png`
show final `070412b`; `tv.png` shows the original viewer layout at `e159871`, and
`vision.png` shows the original window at `e2677f3` before subsequent behavior repairs.
The final device build emitted a harmless unsupported watchOS `cdylib` warning;
the required Rust static library built and linked successfully.

## Remaining limitations

ARM64 Apple builds only; older arm64_32 watches are not configured. Watch app packaging
is a local ad-hoc companion build; physical installation requires owner provisioning.
Paired physical WatchConnectivity delivery, physical Digital Crown/Siri Remote,
Vision eye/hand input, pen feel and accessibility speech are not certified by these checks.
Vision is a 2D window editor, with no immersive or spatial document editing.

No cloud synchronization, autosave/process-death recovery or release signing. Cache
availability is not durable storage, especially on tvOS. Documents and path geometry
are processed synchronously on the UI actor; large-document/watch memory and drawing
latency are unmeasured. The original cross-platform limitations still apply.

## Reproduction

```sh
./scripts/check
./scripts/build-apple-family simulator
./scripts/check-apple-viewers
./scripts/run-ios <iPhone-UUID> --integration-test
./scripts/run-visionos <Vision-UUID> --integration-test
./scripts/run-tvos <TV-UUID> --integration-test
./scripts/run-watchos <Watch-UUID> --integration-test
```

Use `device` instead of `simulator` for compilation intended for owner provisioning.
The simulator Watch app additionally accepts `--preview-sample` for a labelled synthetic
copy used to inspect small-screen layout; that is not evidence of a received phone transfer.
