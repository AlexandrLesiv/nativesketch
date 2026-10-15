# Native Sketch

Editable vector sketching prototypes with shared Rust document, drawing, geometry,
selection, history, and persistence. Native controls surround a custom canvas on each
platform. This is the first-stroke milestone, not a production drawing application.

| Platform | Interface | Build / checks |
| --- | --- | --- |
| macOS | Swift / AppKit toolbar, menus and inspector | `scripts/run-macos`, `scripts/check` |
| iOS / iPadOS | Swift / UIKit, touch tools and document picker | `scripts/build-ios simulator`, `scripts/build-ios device` |
| visionOS | Swift / UIKit 2D editor | `scripts/build-visionos simulator` |
| tvOS | UIKit view-only document viewer | `scripts/build-tvos simulator` |
| watchOS | Independent SwiftUI viewer, optional iPhone source | `scripts/build-watchos simulator` |
| Android / compatible ChromeOS / Googlebook | Adaptive Kotlin / Material Views; ARM64 + x86-64 | `scripts/build-android`, `scripts/check-android` |
| Windows | C# / WinUI 3 | `scripts/check-windows.ps1` on Windows |
| Linux | Rust / GTK4 + libadwaita | `scripts/check-linux` |
| Browser | TypeScript / semantic HTML + Rust/Wasm | `scripts/run-web`, `scripts/check-web` |

Apple chrome uses standard native controls and current SDK styling; Android uses
Material Views. Actual platform runtime verification varies: read the
[platform evidence](docs/evidence/005-native-platforms.md) before accepting a port.
[Android desktop evidence](docs/evidence/008-android-desktop.md) records emulator results
and remaining ChromeOS/Googlebook hardware and interface checks.
GTK has been built on this Mac; that does not certify Linux. Windows C# compilation
and its Rust interop check do not certify a packaged, running Windows application.

Apple TV and Apple Watch are **view only**. TV opens explicit HTTPS document links;
Watch opens cached copies, prefers iPhone and supports a configured server fallback.
Production storage/authentication is not connected. Vision Pro supports editing in a native 2D
window. Build the complete Apple family, including the embedded Watch companion, with
`scripts/build-apple-family simulator` (or `device`, requiring owner provisioning).
See [Apple-family evidence](docs/evidence/006-apple-device-family.md) and
[Watch source verification](docs/evidence/007-watch-document-sources.md) for support limits.

## What works in this milestone

On editing-capable devices, draw with optional device pressure, select one stroke, uniformly scale, delete,
undo/redo, pan/zoom, and save/open portable `.nsketch` documents. Rust owns all
editable content and validation. Native hosts adapt input, viewports, controls and
file permissions. Mobile, browser and GTK two-contact navigation cancels temporary
ink before panning/zooming. Windows cancels a second pointer but currently uses
Space-drag/scroll for navigation.
Failure to open preserves the current drawing; explicit replacement is guarded.

Platform path painters are provisional: CoreGraphics, Android Canvas, WinUI paths,
Cairo and Canvas2D consume Rust geometry. **Production renderer selection remains
OPEN** for task 002's comparable Skia/Metal and Vello experiments. No GPU or physical
pen latency claim is made.

## Build and run

Start on macOS with pinned Node 24.12.0, Rust 1.99.0, Swift 6+ and a macOS SDK:

```sh
npm ci --ignore-scripts
node scripts/check.mjs
node scripts/run.mjs --platform macos
node scripts/check.mjs --core
node scripts/board.mjs
```

See prerequisites and behavior in [iOS](apps/ios/README.md),
[Android](apps/android/README.md), [Windows](apps/windows/README.md),
[GTK](apps/linux/README.md), and [browser](apps/web/README.md) guides.
Mobile device signing and store distribution are not configured. Build outputs are
ignored under `artifacts/`, `target/`, and platform build directories.
See [Node commands and verification gaps](docs/quality/commands.md). The legacy platform
commands above remain available until their native-host replacements have verified parity.

## Limits and approval

Single-document editing, single-stroke selection and uniform scale. No layers UI,
image import, autosave/crash recovery, production GPU renderer or release packaging.
File operations and full-scene path painting are synchronous; large-document
responsiveness is unmeasured. Browser downloads cannot confirm a completed filesystem
write, so Download copy keeps the unsaved indicator and never implies durable saving.
Physical tablets, screen-reader speech and unavailable OS hosts need separate checks.

Local task branch changes are ready for owner review only after the evidence report's
checks. The owner approves merging and distribution; agents do not merge into main.
No remote repository, CI service, license or release identity is configured.

- [Native platform evidence](docs/evidence/005-native-platforms.md)
- [Original macOS milestone evidence](docs/evidence/001-first-stroke.md)
- [Native interactions](docs/design/native-platforms.md)
- [Architecture](docs/architecture.md)
- [Renderer boundary](docs/decisions/0002-first-stroke-rendering.md)
- [Quality](docs/quality.md) · [Workflow](docs/workflow.md) · [Tasks](tasks/README.md)
