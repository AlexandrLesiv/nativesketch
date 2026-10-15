# Task 017 macOS native E2E foundation

Implementation starts at `4c8355f44fe7a59197d79b131cf13b9448e4bb59` in the isolated
`codex/macos-e2e-foundation` worktree. This is a compiled driver with native execution
**blocked**, not completed native acceptance. No physical pen or latency measurement ran.

The standalone XCTest UI target and three small Swift test files compile with Xcode 27.0
(27A266a), macOS SDK 27.0, on Mac14,10 arm64 running macOS 27.0.1 (26A434). The Node
preparation command builds the actual Swift/Rust application and packages an ad-hoc-signed
copy with a source/worktree-specific bundle identifier. This prevents XCTest launch/quit from disturbing an
ordinary Native Sketch process or its unsaved documents. Disposable files live within the
new per-run evidence directory. Native controls keep their spoken labels and gain stable
canvas/width/color identifiers through a 10-line accessibility helper; the existing large
Swift file does not grow. Its extraction remains task 019.

The intended executable case is
`xctest:NativeSketchUITests:EditorJourneyTests/testDrawSaveReopen`. It uses actual XCUI
keyboard/pointer actions through AppKit, native width and color controls, a saved-document
read-only oracle (RGBA, width, mouse provenance, absent pressure, sample ordering and straight
trajectory), interior red-pixel assertions, normal Quit/new PID/relaunch/Open, and Save As
followed by exact byte comparison. These assertions target BRUSH-001 and FILE-001. Their
selectors and behavior remain unproven until the real journey executes. No planned mapping
should be presented as a passing or discovered native test.

## Execution evidence and blocker

An actual initial XCTest invocation failed before test execution with
`com.apple.LocalAuthentication Code=-1004`, “Can't authenticate off console”. The execution
user differs from the active console owner. The subsequent enumeration-only invocation
returned exit zero but contained the same setup errors and only a suite name, so no native
executable test was discovered. The driver now checks console ownership before invoking
XCTest and rejects enumeration errors regardless of process exit status. User switching
and authorization remain human actions; no account, trust or privacy settings were changed.
The lead requested the active user session; dependent UI attempts remain pending.

Ignored artifacts under `artifacts/evidence/macos-prepared-2/` record the successful app and
test builds, source/diff identity and binary SHA-256 values. `artifacts/first.xcresult` and
initial local logs preserve the off-console failure. The preparation run deliberately
returns nonzero, leaves discovery/results empty and writes `blocked.json`. It neither runs
UI nor turns a successful compile into an E2E result. Evidence must be rerun at the final
integrated source revision before closing task 017.

## Verification and limits

- Expanded XCTest target: build-for-testing passed; app build/package passed.
- New Swift files: strict swift-format passed; all new source files are below 200 lines.
- Node runner: TypeScript checking and type-aware Oxlint passed; Oxfmt passed.
- Five executable Node tests passed: enumeration errors despite exit zero, unique actual
  case discovery, system setup failure exclusion, assertion/skip preservation, and failed
  runner finalization invalidating a nominal Passed case.
- Runtime source identity is checked after building and before results publication; changed
  tracked or untracked source rejects stale evidence. App and test executable hashes differ
  explicitly. Failed/skipped/blocked outcomes survive imported result envelopes. Explicit
  attempt numbers and new output directories preserve retries.
- Native full journey, focused-field shortcuts, actual color/file panel selector behavior,
  accessibility speech, pen pressure/palm rejection, display changes and recovery: not run.

See [runner instructions](../../apps/macos/UITests/README.md). Other platform E2E drivers are
not claimed by this macOS foundation. No renderer, persistence or editing semantics changed.

Same-source retries in this checkout retain the isolated bundle identifier so ad-hoc signing
produces the same executable identity. Two separately copied/signed app bundles produced
the identical SHA-256 in a non-UI reproduction. Source changes create a new isolated identity.

## Designer oracle follow-up

The follow-up rejects partial/translated ink that the original test could miss. Each of
three interior patches independently requires more than 20 red pixels; their counts are
no longer summed. The journey invokes native File > New after layout to establish the
existing native viewport reset (zoom 1, offset (80,80)), captures AX canvas logical size,
and rejects unexpected frame changes. Saved endpoints must match normalized pointer
positions `(0.3,0.4)` and `(0.7,0.6)` converted through that documented viewport, within two
logical points for OS pointer rounding. This follows `Canvas.isFlipped`, `document(_:)`
and `newDocument(_:)` in the actual adapter; screenshots do not establish the transform.
The reopened canvas must retain the same logical size before its three-patch check.

A standalone pure Swift regression harness exercises the same predicate implementation:
one-patch-only and below-threshold counts fail; complete counts pass; translated, scaled,
empty and nonfinite endpoints fail; intended endpoints and one-point rounding pass.
Compile/run without UI:

```text
DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer xcrun swiftc -module-cache-path artifacts/oracle-module-cache apps/macos/UITests/StrokeExpectation.swift apps/macos/UITests/Regression/OracleRegression.swift -o artifacts/oracle-regression
artifacts/oracle-regression
```

The harness passed all ten preconditions; strict Swift formatting and the expanded
XCTest target build passed. The first bare CLT compile encountered cache permission/SDK
resolution errors; explicit Xcode plus a workspace module cache resolved it. These are
oracle regression/build results only. Native execution remains blocked and unclaimed.

## First active-console attempt

On clean `5aef8de8cdb65165f2f74e0b4f5876180952a457`, the real Xcode runtime enumeration
succeeded and discovered the intended executable case. The isolated signed app launched;
XCUI opened the native inspector, entered width 12, opened the Colors panel and selected
RGB Sliders. The journey then failed at the hex text-field query. The exported actual AX
hierarchy exposes that field with identifier `hex`; “Hex Color #” is a separate static
label. The selector now uses the observed identifier. This is a test-driver correction,
not a product behavior change. It has been compiled but not yet rerun through native UI.

Attempt 1 is preserved unmodified in the native acceptance worktree under
`artifacts/evidence/native-attempt-1/`, including runtime discovery, failed normalized
result, execution log, `journey.xcresult`, screen recording, screenshot and AX attachments.
App executable SHA-256:
`043c3ec550510a2a272ec70880b5a1a5236be20646ca35a2b19e1195076b11b3`.
Test executable SHA-256:
`ef1a3cb2e5c2e784e5a292208b0ade5132658cc97221ff36036840b0be9af5b3`.
No drawing/save/reopen acceptance passed. This supersedes the earlier discovery blocker,
not the outstanding full-journey gap. The app was terminated by test teardown.

The owner subsequently requested background work; further foreground UI attempts are
paused. Native XCTest needs an active desktop and can move focus/pointer. Compiling,
formatting and pure oracle checks continue without opening the app. Resume actual UI only
in an agreed foreground slot or a dedicated native host; hidden/minimized/off-console
execution cannot be substituted for this journey's visible native behavior evidence.

## Background-default execution policy

The owner clarified that native tests should run when verifying relevant commits or while
developing UI tests, rather than on every background check. The CLI now defaults to
background preparation; native discovery/input requires explicit `--interactive`. The
`--prepare-only` alias remains; specifying both flags fails before any build or launch.
No Git hooks or global settings were installed. Eight Node tests, including default/alias/
interactive/conflict/retry parsing, pass with type-aware lint, formatting and type checks.
An actual default invocation built the app and XCTest target and ran pure Swift oracles,
then returned nonzero with empty discovery/results and no enumeration/execution artifacts,
as intended for incomplete native acceptance. Evidence is under
`artifacts/evidence/background-default-check/`. The hex selector correction also compiled.
The next explicit interactive run will verify this frozen correction; prior failed attempts
remain untouched. Native execution is authorized within this test-development scope.

## Mouse-route correction after attempt 2

On clean `851a6c7848eb1e2d97760157bf9bef77a57773c0`, attempt 2 entered the requested
red hex value successfully but committed no stroke. Its preserved execution log shows
`press(forDuration:thenDragTo:)`. Inspection of the installed Xcode SDK's
`XCUIAutomation.framework/Headers/XCUICoordinate.h` establishes that this belongs to
`XCUICoordinateTouchEvents`; the correct AppKit route is the distinct
`click(forDuration:thenDragTo:)` in `XCUICoordinateMouseEvents`. The test now uses that
mouse API. The failed run's declared intended mouse route is not evidence of delivered
AppKit mouse input; the zero-stroke failure and original artifacts remain unchanged.

A frame extracted from the existing recording shows the inspector closing immediately
before the gesture. This does not establish an additional popover bug. The driver now
explicitly waits for Colors and inspector closure before drawing, and preserves a pre-drag
canvas screenshot. It does not inject domain commands or create an extra stroke for focus.
The lead independently checked the SDK distinction before authorizing this next repair.

## Native Save selector correction after attempt 3

On clean `bc4a3f4`, the native mouse route committed one stroke and all three independent
red-pixel patches passed. Attempt 3 then reached the actual Save panel and failed at an
outdated combo-box query. Its AX attachment identifies the Go To sheet as `GoToWindow`
and its text field as `PathTextField`. The compact Save panel independently exposes
`saveAsNameTextField`. The helper now selects/replaces those native field values, navigates
to the parent directory and enters the new filename separately for Save. Open retains
the existing-file path route. Sheet dismissal and Save enablement are checked before
proceeding. The lead verified these observations before authorizing the next attempt.
All failed attempt 3 artifacts remain under `artifacts/evidence/native-attempt-3/`.
No saved-file or full journey pass is inferred from its successful drawing segment.
