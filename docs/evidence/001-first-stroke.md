# Task 001: first editable stroke evidence

Date: 2026-10-14. Owner merge and physical drawing acceptance remain pending.
Base: `b63a22306ba3969b651a69ece801f1433769687c`.
Task branch: `codex/first-stroke`; main was not changed.

## Implementation

Rust owns editable actual samples, optional pressure, brush style, stable IDs, selection,
uniform scale transactions, undo/redo, validated persistence, and path geometry. Swift
owns AppKit controls, event capture, viewport conversion, dialogs, and presentation.
The C ABI uses an opaque serialized editor handle, caller-owned path copies, status
codes, and panic containment. A poisoned editor is discarded and cannot keep a stale
Save destination when recovering.

Pen pressure uses width × (0.2 + 0.8p); mouse samples retain missing pressure and nominal
width. Uniform resize scales geometry and width together. One stroke/transform/delete
is one history operation. Escape/deactivation cancel temporary operations. Save uses a
same-directory temporary file, sync, and atomic replacement; loading validates before
publishing state.

The renderer is a disposable CoreGraphics painter of Rust geometry. Production renderer
selection is still OPEN; task 002 has not been completed and must precede a permanent
backend commitment. See decision 0002.

## Automated verification

- `./scripts/check` passed at `d3d319e` and `13e18d5`: board, formatting, clippy with warnings
  denied, 12 core tests, 2 FFI tests, actual SwiftPM build, and Swift/Rust integration.
- Release build, `--integration-test`, and `--native-smoke-test` passed at `1bf40a9`.
  Native smoke drives synthetic AppKit mouse events through the real canvas methods;
  it checks cancellation, selection/resize/history, viewport/window mapping, and the
  failed recovery-open Save-destination regression. This is not physical-device input.
- Core tests include the named `synthetic-pressure-v1.csv` replay, actual versus predicted
  samples, corrections and atomic invalid batches, topmost selection, resize cancellation,
  no-op history, exact undo/redo, dirty revision identity, roundtrip geometry/style,
  checksum/truncation rejection, and injected interruption before atomic replacement.
- FFI tests cover null/error handling, caller-owned copies/capacity, real edit flow, and
  caught panic. Swift integration checks poison disposal/recovery and exact geometry/style
  roundtrips through the linked C ABI.
- Initial full Swift build failed on nonisolated destruction of an opaque pointer;
  an isolated deinitializer fixed it. Standalone typechecking alone had missed the error.
- Build emits existing SDK search-path linker warnings for absent CommandLineTools
  Developer library/framework directories; linking, launch, and integration succeed.

Final tested code: `c65b3335988c4b8b26905faf1a99720ff6703736`.
`./scripts/check`, release build, release `--integration-test`, and release
`--native-smoke-test` all passed on this revision. Final native smoke additionally
asserts focus, full text selection, and persistent invalid-scale guidance. Local logs:
`artifacts/evidence/check-final.txt`, `release-build.txt`, `integration-final.txt`, and
`native-smoke-final.txt`. Later report/task-status commits change documentation only.

## Native observations

Lead CUA inspection at `d3d319e` on the actual Mac observed native Pen/Select, color,
width, and scale controls; mouse drag drawing; Cmd-] selection; numeric 150% resize;
Cmd-Z restoration; native Save panel; New/Open and visibly identical reopened stroke.
All documents used for these checks were disposable fixtures under `/private/tmp`.
The Mac locked during the first reopen attempt; after owner unlock the check completed.
No global appearance, keyboard, security, or trust settings were changed.

Designer running review at `1bf40a9` reproduced mouse drawing, keyboard selection,
Tab-based scaling, Undo, corner-handle resizing, native Zoom/large layout, and the dirty
Quit prompt's Cancel behavior. The designer identified a missing focus-ring redraw and
incomplete invalid-input selection/announcement. These were repaired in `13e18d5`, with
validation feedback ordering corrected afterward.

Final designer CUA review at `c65b333` passed compact 640×420 content layout,
app-local dark appearance, repaired focus ring, invalid scale focus/selection and
persistent message, corrupt Open alert with drawing retained, keyboard zoom/Actual Size,
Escape deselection, and Pen switching. No remaining blocker was observed in the exercised
UX. Dark override applied to the window; native alerts followed the system light appearance.
VoiceOver speech, pinch input, and changed OS accessibility settings were not tested.
The lead then confirmed the final release remained running and selected its disposable
stroke for owner inspection.

CUA screenshots/AX observations are in the task conversation. Automated native checks
are complementary and do not replace the running-app observations.

## Provisional CPU measurement

Measured revision: `1bf40a9894a60c3082c6c4f41bb5ae467de39fad`, release Rust and Swift builds.
Machine: MacBook Pro Mac14,10, Apple M2 Pro (12 CPU cores, 19 GPU cores), 16 GB RAM.
OS: macOS 27.0.1 build 26A434. Rust/cargo 1.99.0; Swift 6.4.
AC power, battery 80%; pmset reported no thermal/performance warning.
Other agent builds, native automation, and tests were stopped during measurement.
Ordinary background desktop processes were not controlled.

Fixture: `Integration.timing`, one growing 120-sample synthetic pressure stroke; ten
warm-up strokes then 100 measured repetitions, clearing the document between strokes.
12,000 incremental append/path-copy observations. Monotonic system-uptime timer; very
short append timings are near timer/measurement overhead and must not be overinterpreted.

| CPU stage | p50 µs | p95 µs | p99 µs |
| --- | ---: | ---: | ---: |
| Append one input sample | 0.083 | 0.125 | 0.250 |
| Query and copy current path commands | 9.042 | 15.083 | 18.750 |

`time -l`: 0.13 seconds wall time, maximum resident set 11,878,400 bytes; peak memory
footprint 6,488,544 bytes for this command process. These are not GUI-process memory.

Reproduce after `./scripts/build-macos release`, with exclusive benchmark host access:

```sh
/usr/bin/time -l ./artifacts/NativeSketch.app/Contents/MacOS/NativeSketch \
  --timing --timing-csv artifacts/evidence/cpu-timing.csv
```

Local raw artifacts: `artifacts/evidence/cpu-timing.csv`, `cpu-timing.txt`,
`cpu-resources.txt`, `power.txt`, and `thermal.txt` (ignored generated evidence).
CSV SHA-256: `7ab8b5e2eb9e01d453dd2cbc8234c57892300299a0eeb0e98a03440262820d9d`.

This tiny CPU-only fixture excludes CoreGraphics painting, GPU work, display scheduling,
missed presentation deadlines, dense documents, sustained workloads, first-use GPU stalls,
and end-to-end latency. Resolution/refresh are not parameters of this headless path-copy
workload. It is not a Skia/Vello comparison or a responsiveness guarantee. No baseline or
threshold was altered. Subsequent fixes touch native focus/error UI only, not timed code.

## Independent review

Fresh `reviewer` specialist examined the requirement and integrated diff, independently
reproduced 12 core + 2 FFI tests at `f474ba0`, and verified that Rust sources were unchanged
at `1bf40a9`. It identified and re-reviewed fixes for:

1. Continuing to use an editor after caught panic: discard once, disable unavailable
   editing, show an honest error, and create fresh state for recovery.
2. Accepting numeric prefixes such as `200junk`: parse the entire finite numeric value.
3. Retaining old document URL after failed recovery Open: clear the destination before
   creating empty recovered state, so Save cannot silently overwrite the prior file.
4. Focus transfer overwriting validation guidance: assign guidance after focus/selection.

Reviewer did not perform native UI or timing work; those were serialized on this Mac.
Final independent assessment at `c65b333`: no outstanding actionable findings after
re-review. Reviewer inspected final logs and recomputed the timing CSV hash, row count,
and percentiles. This review is separate from designer runtime inspection and does not
constitute owner merge approval or exhaustive proof of correctness.

## Remaining limitations

- No physical tablet/pen feel, pressure calibration, orientation, coalescing, or pen-to-screen
  latency verification. Native input supports primary mouse/tablet strokes only.
- No production renderer or task 002 comparison; full-scene path rebuilding/copying is
  uncached. Dense-document responsiveness and GPU behavior are unestablished.
- Synchronous file I/O outside gestures; 64 MiB loader limit, bounded stroke/sample counts,
  early local format, no migration beyond rejecting unsupported versions. Directory sync
  is best effort; full power-loss durability and autosave/crash recovery are not claimed.
- One document window, one selected stroke, uniform resize only. No layers UI, image import,
  export, textured brushes, cloud, Windows app, or distribution signing.
- Tested on this Mac/SDK; the provisional macOS 14 minimum and other hardware were not tested.

## Owner decision

Pending human drawing/merge approval. No merge into main or release publication performed.
