# Native editor UI journey

This standalone XCTest UI target launches a built app copied and signed with a unique
bundle identifier for this source/worktree. It must never attach to or terminate the user's ordinary
Native Sketch process. No app domain commands or injected Rust replay are used.

```text
npm ci --ignore-scripts
node scripts/test-e2e.ts --platform macos
node scripts/test-e2e.ts --platform macos --interactive --attempt 2 --output artifacts/evidence/mac-retry
```

The default builds/packages in the background without executable discovery or UI execution.
`--prepare-only` remains an explicit alias. Add `--interactive` only in an agreed foreground
session to execute the journey; combining both flags is rejected. Preparation
returns nonzero and writes a setup diagnostic because preparation is not an E2E pass.
The execution user must own the active, unlocked macOS console. Full Xcode is required;
the runner uses process-local `DEVELOPER_DIR` (default `/Applications/Xcode.app/Contents/Developer`).
It does not change system Xcode selection, switch users or grant permissions.
Run native UI tests for relevant commit verification or while developing UI tests, rather
than every background check. No automatic Git hook is installed.
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

The original five journeys cover drawing/save/reopen, real Save As keyboard delivery,
one-stroke Undo/Redo, field shortcut priority and incomplete-width refusal/restoration.
Their historical passes do not certify the new versioned runtime. New source-authored
journeys below require fresh executable discovery and execution before coverage is recorded.

| Test group | Intended acceptance and decisive observation |
| --- | --- |
| AsyncSaveTests | NATIVE-ASYNC-SAVE-001 / NATIVE-FIELD-REFRESH-001: admitted A versus newer B bytes; unfinished text/focus/undo and corrected saved width |
| AsyncSaveAsTests | NATIVE-ASYNC-SAVEAS-001 / NATIVE-SAVE-FAILURE-001: binding before acknowledgement, fresh retry chooser, directory-sync warning versus actual bytes |
| NativeNavigationTests | NATIVE-NAVIGATION-001 / NATIVE-CLOSE-QUIT-001: cancellation/discard, pending save drain, invalid field blocking Quit |
| NativeOpenTests | NATIVE-OPEN-ORDER-001 / NATIVE-STALE-EFFECT-001 / NATIVE-ASYNC-FOCUS-001: ordered Open, corrupt input retention, cancelled resolution preserving later drawing/focus |
| NativeRecoveryTests | NATIVE-RECOVER-COPY-001 / NATIVE-RECOVER-MANAGE-001: real checkpoints, Later, dirty unbound copy, corrupt-newest fallback and selected discard |
| NativeRoutingTests | NATIVE-ROUTING-001 / NATIVE-SAVE-FAILURE-001: menu/shortcut/control equivalence and a real filesystem failure deferred behind another sheet |
| NativeSaveIntegrityTests | Ordinary Save retains the old binding during/after failed Save As; failed overwrite preserves last-good bytes |
| NativeDecisionTests | New/Open choice matrix, cancelled Close during a held write, and failed Save during Quit |
| NativeReadIsolationTests | Oversized input, held validated reads, obsolete resolution failure and closed-window late delivery |
| NativeRecoveryDecisionTests / NativeRecoveryManagementTests | Recovery over dirty edits, actual capacity exhaustion and discard racing a newer real checkpoint |
| NativeFieldCompletionTests / NativeFocusRestorationTests / NativeSheetFocusTests | Nonzero selection across save completion, cancelled Save As, Recovery Later and failed-save cancellation |
| NativeInspectorDismissalTests | Inspector Escape returns working native canvas P/V shortcuts and drawing |
| NativePoisonClosureTests | Explicit caught Rust panic, cancelled native Close/Quit, confirmed exit and unchanged last-good file reopen |

The runner explicitly builds DEBUG with `native-test-controls`; ordinary/release builds
do not include this API. Selected tests arm one real-worker stage using a disposable empty
directory. The worker's `reached` marker proves the stage; the test's `release` marker only
allows work to proceed. No document mutation or successful receipt is injected. Stage IDs
come from `sketch_test_v1.h`: resolution=1, before-create=2, before-replace=5, directory-sync=7.
After-read-validation=8 holds the real decoded Document/Recovery before candidate publication;
New and invalid reads do not consume it. An explicit `--test-poison-on-close` launch argument
in DEBUG + NATIVE_TEST_CONTROLS builds causes one panic inside the real caught C boundary at
the first native Close/Quit. It does not set Swift/Rust poison flags directly. The separate
C test API is absent from ordinary release builds; the integration gate checks both symbols.
Other cases use native input and actual file/recovery I/O without arming a gate. Read-only
DEBUG AX observations expose actual focus and runtime quiescence, never force them.

Executable discovery and results must be imported independently: adding or compiling these
tests does not close their runtime acceptance. Remaining specialist checks include active-gesture
resize/deactivation through real OS events, accessibility speech, full keyboard traversal,
display scale and physical device input. A disposable CGEvent/AX helper is a different input
route from XCUIAutomation and must have separate source/binary/fixture evidence.
The pristine runtime/Swift shutdown-ordering smoke and actual Rust poisoned-detach tests
are complementary lower-level evidence, not poisoned-engine native E2E.

Same-source retries in this checkout retain the isolated bundle identifier so ad-hoc signing
produces the same executable identity. Two separately copied/signed app bundles produced
the identical SHA-256 in a non-UI reproduction. Source changes create a new isolated identity.

## Oracle regressions

Both background preparation and explicit `--interactive` runs compile and execute the pure Swift pixel/endpoint
regression harness before building XCTest. A failed oracle check stops preparation and is
recorded as blocked setup, never as native execution. Logs live in the run directory.
