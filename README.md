# Native Sketch

A macOS-first vector sketching prototype with a shared Rust editor and native AppKit UI.
Task 001 is under implementation review; this is not a production drawing application.

## First-stroke milestone

- Draw with a mouse or pressure-capable pen; mouse width is fixed and absent pressure
  remains absent in the document. Pen width uses nominal width × (0.2 + 0.8 × pressure).
- Select strokes by clicking or with Edit → Select Next/Previous Stroke. Resize uniformly
  using corner handles or Scale percent. Geometry and width scale together.
- Undo/redo drawing, resize, and deletion; Escape cancels an active operation.
- Pan with Space-drag/scroll and zoom with pinch or View commands.
- Save/open versioned `.nsketch` documents with geometry/style preserved. Atomic replacement
  protects the previous file on save failure; unsaved replacement/close is guarded.

Native controls surround a custom white-paper canvas. The temporary CoreGraphics painter
consumes Rust paths. **Production renderer selection remains OPEN** pending task 002's
Rust-Skia/Metal versus Vello comparison; no GPU-performance winner is claimed.

## Build and run

Prerequisites: Rust 1.85+ with cargo, rustfmt, and clippy; Swift 6+ and a macOS SDK;
Python 3.9+ for task-board checks. macOS 14 is the provisional minimum. No external Rust
or Swift packages are needed.

```sh
./scripts/check            # board, Rust fmt/clippy/tests, Swift build + integration
./scripts/run-macos        # build and launch a development .app
./scripts/board            # task status and dependencies
./scripts/check --core     # Rust and metadata checks on non-Mac hosts
./scripts/build-macos release
./artifacts/NativeSketch.app/Contents/MacOS/NativeSketch --integration-test
./artifacts/NativeSketch.app/Contents/MacOS/NativeSketch --native-smoke-test
./artifacts/NativeSketch.app/Contents/MacOS/NativeSketch --timing
```

Native smoke uses synthetic AppKit mouse events on a real Mac. Timing is a provisional
CPU workload, not GPU, presentation, or physical input latency. Run timing alone after
other agent builds and native automation stop. `--appearance light` / `--appearance dark`
inspect app-local appearance without changing system settings.

## Scope and limitations

Single-document window; single-stroke selection and uniform resize. File I/O is synchronous
outside drawing gestures. The current path painter is deliberately uncached; large-document
responsiveness is not established. There is no autosave/recovery, production renderer,
Windows app, image import, layers UI, distribution signing, or physical tablet certification.
The versioned file schema is an early local format, not a compatibility promise for release.

Generated output and local evidence belong under ignored `target/`, `.build-cache/`, and
`artifacts/`. Tests use disposable synthetic documents. No baselines or thresholds should
be weakened to pass checks.

## Project workflow

The lead integrates scoped specialists, with design before UI implementation and a fresh
review before owner acceptance. Local task-branch commits are allowed; the owner approves
merges and release distribution. No cloud repository, CI provider, remote branch protection,
release identity, or open-source license is configured.

- [First-stroke design](docs/design/first-stroke.md)
- [Architecture](docs/architecture.md)
- [Provisional renderer boundary](docs/decisions/0002-first-stroke-rendering.md)
- [Quality and evidence](docs/quality.md)
- [Agent workflow](docs/workflow.md)
- [Tasks](tasks/README.md)
