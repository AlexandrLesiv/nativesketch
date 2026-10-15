# Native editor UI journey

This standalone XCTest UI target launches a built app copied and signed with a unique
bundle identifier for this source/worktree. It must never attach to or terminate the user's ordinary
Native Sketch process. No app domain commands or injected Rust replay are used.

```text
npm ci --ignore-scripts
node scripts/test-e2e.mjs --platform macos
node scripts/test-e2e.mjs --platform macos --attempt 2 --output artifacts/evidence/mac-retry
```

Use `--prepare-only` to build/package without executable discovery or UI execution. It
returns nonzero and writes a setup diagnostic because preparation is not an E2E pass.
The execution user must own the active, unlocked macOS console. Full Xcode is required;
the runner uses process-local `DEVELOPER_DIR` (default `/Applications/Xcode.app/Contents/Developer`).
It does not change system Xcode selection, switch users or grant permissions.
Reserve the native display session exclusively; do not cover/minimize the app or use the
pointer during execution. XCTest may require interactive system authorization.

The intended test ID is
`xctest:NativeSketchUITests:EditorJourneyTests/testDrawSaveReopen`. Only runtime Xcode
executable enumeration can register it as discovered; source code alone cannot.
The journey targets BRUSH-001 and FILE-001: native width/color controls, mouse drag,
interior colored-pixel checks, native Save, read-only document geometry oracle, normal
Quit/new process, native Open, restored pixels and native Save As/byte equivalence.
Native color-panel/file-panel selectors require validation on each supported OS version.
This is simulated OS mouse/keyboard input, not a physical pen, pressure or latency test.

Every run requires a new output directory and preserves app/test binary hashes, exact
revision plus dirty/untracked source digest, build/runner logs, discovery/results JSON,
source-change rejection and the xcresult bundle with screenshot/AX-tree attachments.
Failed runner finalization cannot become a passed imported result. Retry attempts retain
separate output directories and monotonically supplied attempt numbers. No automatic
retry hides failures. `durationMs` is runner execution wall time including startup/result
collection, not input/render/physical latency. Setup failure before discovery leaves both
envelopes empty and writes `blocked.json`; it does not fabricate an executable test.

Remaining scenarios: native text-field undo/redo focus, history, selection/cancel,
resize/display scale, accessibility speech, failure recovery and physical device input.
These remain coverage gaps; this one journey does not certify their invariants.

Same-source retries in this checkout retain the isolated bundle identifier so ad-hoc signing
produces the same executable identity. Two separately copied/signed app bundles produced
the identical SHA-256 in a non-UI reproduction. Source changes create a new isolated identity.
