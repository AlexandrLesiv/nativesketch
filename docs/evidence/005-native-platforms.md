# 005: Native platform implementation evidence

Date: 2026-10-14. Branch: `codex/native-platforms`, based on first-stroke `59246c3`.
Main remains `b63a223`; nothing has been merged or published. Platform acceptance and
release distribution remain the owner's decisions.

## Shared implementation and review

Rust owns strokes, pressure semantics, geometry, selection, transforms, history, document
validation and portable bytes. ABI 2 adds snapshot/import/revision acknowledgement;
raw Wasm protocol 1 reaches the same engine. Native UI choices are AppKit, UIKit,
Material Views, WinUI 3, GTK4/libadwaita, and semantic HTML. Platform painters only consume
Rust paths. Task 002 is **not completed** and no permanent renderer has been chosen.

The configured designer defined interactions before implementation. Rust/Swift byte
contracts were agreed before concurrent implementation in isolated worktrees. At most
two implementation writers were active. Specialist and worker changes were integrated
by the lead. A fresh reviewer independently inspected the changes and reproduced checks.

## Verification identities

- `46e56d8`: integrated `scripts/check` passed: task board, Rust formatting/clippy,
  17 Rust tests, macOS build and Swift/Rust integration. Browser actual-Wasm checks and
  GTK formatting/clippy/adapter test/native build also passed on this integrated source.
- `07aadcc`: actual shared C# `Engine.cs`/Rust dylib conformance passed; WinUI C# source
  compilation passed against the real Microsoft.WindowsAppSDK 1.8.260921001 packages.
- `979ec3c`: final browser/GTK designer repairs compiled and passed applicable checks.
  The fresh reviewer found no remaining actionable issue in this reviewed diff and
  independently reproduced TypeScript compilation and GTK build/adapter checks.
- Android and UIKit app source is unchanged after `46e56d8`. Android build and lint
  passed (0 errors, 13 warnings); phone/device UIKit builds passed. Final Android,
  iPhone and iPad runtime replay logs are under `artifacts/verification/`.

## Platform coverage

| Platform | Build / automated evidence | Running interface evidence | Limits |
| --- | --- | --- | --- |
| macOS | Debug build; 17 core/FFI/Wasm tests; Swift/Rust integration | Designer drew, selected, scaled, undid; native save/reopen and corrupt-open retention; dirty Cancel/close; field shortcut isolation; compact 640pt dark toolbar/inspector retest passed | Physical pen and screen-reader speech not tested |
| iOS | arm64 simulator and arm64 device builds; iPhone 18 Pro iOS 27 replay passed | UIKit adapter executes in simulator | Device build needs provisioning; simulator UI inspection unavailable; physical Pencil untested |
| iPadOS | Same UIKit build; iPad Pro 11-inch M5 iOS 27 replay passed | UIKit adapter executes in simulator | Visual/touch/device/VoiceOver checks unverified |
| Android | ARM64 APK; lint 0 errors; real JNI/input/document replay | API 37 emulator replay covers second contact cancellation and actual Back dispatcher → native Cancel retaining dirty bytes | Material UI visually uninspected; API 26 fallback linted, not device-run; physical stylus/TalkBack untested |
| Windows | Same C# wrapper runs against Rust on Mac; WinUI C# compile target passes | None on Windows | Full packaging requires Windows `mt.exe` / `MakePri.exe`; actual Windows build, launch, DPI, picker, pen and Narrator checks not run |
| Linux / GTK | GTK4 4.24.1/libadwaita 1.10.0 build, clippy, adapter test on Mac | GTK window launched on Mac; native tool buttons respond; synthetic mouse cancellation confirmed | Mac GTK execution does not certify Linux Wayland/X11, portals or packaging; mouse-drawing verification is blocked by the automation event stream (details below) |
| Browser | TypeScript compile and actual Rust/Wasm adapter/lifecycle regression tests | Lead drew, selected, applied 150% scale, used Space on Undo, tested Discard→new drawing→New→Escape retention, and 390px toolbar scrolling | No physical touch/pen or screen-reader speech; offline cache is implemented but cold/offline lifecycle was not separately certified |

The browser screenshot is `artifacts/verification/browser.png`. Replay logs are
`android-replay.txt`, `ios-phone-replay.txt`, and `ios-ipad-replay.txt` in that directory.
Generated evidence is ignored local output; this report records the reproducible cases.

## Repair evidence

Review found and repaired rotation/history retention, bounded imports, stale save
acknowledgements, mobile second-contact cancellation, JNI large-ID grouping, Android
API compatibility and Back data-loss risk, Mac compact toolbar overflow, native menu
availability, browser stale modal return values, asynchronous import races, release
samples, Wasm failure states, and keyboard focus interference. Android's native Back
regression invokes the actual dispatcher and the dialog's Cancel button, then checks
that the dirty document bytes are unchanged. GTK/browser source fixes were independently
reviewed after each substantive batch.

Final designer review of `979ec3c` found no new visual blockers in the browser screenshot.
UIKit compact menu and Android Material touch targets were source-reviewed; these do
not constitute mobile visual acceptance.

### GTK runtime inspection limitation

The GTK window and native tool buttons ran on this Mac. Instrumented event tracing
showed that the automation tool's drag sends `MotionNotify` without `BUTTON1_MASK`
between press and release. GTK cancels that sequence, and Rust correctly discards the
temporary stroke. A click also received unrelated zero-time pointer motion before its
release. No workaround was added that would weaken real lost-button cancellation.
The diagnostic instrumentation was removed; final source remains `979ec3c`.
`artifacts/verification/gtk-input-trace.txt` retains the short trace. Real mouse drawing
and Linux-native event delivery remain unverified, rather than marked as a passed UI test.

Designer runtime review is strongest on AppKit. Other platforms received source review;
lead browser interaction is separate evidence. Neither standard widgets nor passing
adapter tests certify accessibility speech, physical input or another OS.

## Reproduction

```sh
./scripts/check
./scripts/check-wasm
./scripts/check-web
./scripts/check-linux
./scripts/build-ios simulator
./scripts/build-ios device
./scripts/run-ios <simulator-UUID> --integration-test
./scripts/build-android
./scripts/check-android
cargo build --locked --release -p sketch-ffi
dotnet run --project apps/windows/InteropCheck -c Release
dotnet msbuild apps/windows/NativeSketch.Windows.csproj -t:Compile \
  -p:Configuration=Release -p:WindowsAppSDKSelfContained=false -v:minimal
```

On Windows use `scripts/check-windows.ps1` for the full x64 DLL, WinUI build and interop
check. The Mac compile-only command deliberately does not claim Windows packaging.
Platform READMEs list SDK requirements. Android's local invocation used JDK bundled with
Android Studio, Gradle 9.3.1, SDK 37, and NDK r30; the NDK was downloaded from Google's
official repository and its published SHA-1 checked. No global developer-directory or
Codex trust settings changed. This host is Apple M2 Pro, 16 GiB, macOS 27.0.1; Rust 1.99,
Swift 6.4, .NET 10.0.107. No concurrent GPU benchmark was run or performance claimed.

## Remaining limitations

First-stroke parity only: one document and single-stroke selection/uniform resize.
No autosave/crash or process-death recovery, layers UI, image import, production GPU
backend, release signing/store packages, or compatibility promise for the early format.
Windows second-pointer input cancels the draft; two-contact navigation is not yet
implemented there (Space-drag/scroll navigation is available). Large-document
I/O/painting is synchronous and unmeasured. Browser Download copy keeps
dirty state because a download request cannot confirm a filesystem write. Android Save
a copy relies on provider completion and cannot guarantee atomicity of a partial new
provider file. Physical pressure devices and end-to-end latency remain untested.

Native UI tooling had severe discovery stalls for Device Hub/Chrome/GTK, even when a
short timeout was requested. No claim of unseen mobile UI approval is made. Linux and
Windows require their own OS acceptance before being presented as verified releases.
