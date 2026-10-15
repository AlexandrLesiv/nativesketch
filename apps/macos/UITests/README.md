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
| NativeGestureCancellationTests | Actual held mouse gesture before resize/deactivation; observed cancellation before mouse-up, exact saved bytes, fresh stroke and single undo |

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
tests does not close their runtime acceptance. The two maintained gesture cancellation cases
combine XCUIAutomation Save/Undo with CGEvent held mouse input, AX native resize and a
test-runner-owned window for deactivation. They require a strictly positive observed interaction
before cancellation, actual size/inactivity readback, and idle before releasing the pointer.
After undoing a fresh stroke, a second Undo must remove the original stroke and Redo restore
its exact saved bytes, excluding a hidden cancelled-draft history entry. Cleanup restores the
test runner's previous activation policy as well as releasing the pointer and owned window.
They prepare their UUID document through the existing native Save helper; the retired ad-hoc
probe failed before drawing while locating the Go To path field. That failure was file-dialog
setup evidence, not evidence of a canvas cancellation defect. Its retained traces remain failures.
These tests fail with an environment diagnostic if the test runner lacks existing AX/event-posting
permissions; they never request permission or change system settings. Preparation/build is not
execution. Runtime results must identify this hybrid synthetic input route, never physical pen.
Remaining specialist checks include accessibility speech, full keyboard traversal, display scale
and physical device input. Gesture acceptance remains pending until these cases actually pass.
The pristine runtime/Swift shutdown-ordering smoke and actual Rust poisoned-detach tests
are complementary lower-level evidence, not poisoned-engine native E2E.

Same-source retries in this checkout retain the isolated bundle identifier so ad-hoc signing
produces the same executable identity. Two separately copied/signed app bundles produced
the identical SHA-256 in a non-UI reproduction. Source changes create a new isolated identity.

## Oracle regressions

Both background preparation and explicit `--interactive` runs compile and execute the pure Swift pixel/endpoint
regression harness before building XCTest. A failed oracle check stops preparation and is
recorded as blocked setup, never as native execution. Logs live in the run directory.

The shared fixture registers framework teardown before launching the subject. Termination
is a separate final block, so an assertion abort or worker-gate cleanup error cannot skip it.
Gesture cleanup is registered later and therefore runs first. Classes already owning an
XCTest `tearDown` keep that owner; no second fixture teardown is registered for them.
The headless Apple XCTest regression deliberately fails one assertion and throws from one
cleanup block. Its runner must observe the expected nonzero result and exact LIFO JSONL
trace before treating the harness check as passed. It never initializes an application or
performs native UI automation; those intentional failures are not native acceptance results.

Every subject launch verifies the actually loaded XCTest bundle against the prepared
external bundle, using required `NATIVE_SKETCH_TEST_BUNDLE_PATH` (absolute `.xctest`) and
`NATIVE_SKETCH_TEST_BINARY_SHA256` (64 lowercase hexadecimal characters). The observed
canonical bundle path, executable path and SHA-256 are attached before comparison. Missing,
malformed or mismatched provenance fails before subject launch, including older raw-app
journeys. Preparation does not prove that the external bundle was actually loaded at runtime.

## Stable runner identity

Preparation freezes the generated host at
`artifacts/e2e-stable-runner/host-v1/NativeSketchUITests-Runner.app` and loads each attempt's
current test bundle separately. Ordinary rebuilds never replace or re-sign this host.
Its manifest records toolchain, architecture, signature and complete bundle fingerprint.
Mutation or a toolchain/architecture change stops preparation and requires explicit host
renewal; do not delete the manifest or overwrite the host to bypass that failure.
The owner may need to grant Accessibility for this stable path once. Tests never grant it.

The run records host/test identities and exports XCTest attachments. Every passing case
must contain matching loaded-bundle provenance; missing evidence cannot become a pass.
Post-run checks reject changes to the host or external test bundle. The frozen embedded
tests are not the intended execution target. External loading and permission continuity
require real native verification in an approved desktop session.
