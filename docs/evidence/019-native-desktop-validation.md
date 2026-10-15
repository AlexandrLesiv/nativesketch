# Native desktop validation — in progress

Host: Mac14,10 arm64, macOS 27.0.1 (26A434), Xcode 27.0 (27A266a),
Swift 6.4, Rust 1.99.0, Node 24.22.0. These are native mouse/keyboard
journeys, not physical pen measurements. The owner reserved the foreground desktop
and subsequently extended that reservation to 2026-10-15 20:10 UTC.

## Preserved runs

| Attempt | Source | Actual outcome |
| --- | --- | --- |
| 12–13 | worker `df804bd` | XCTest automation initialization timed out before discovery; no cases executed |
| 14 | worker `df804bd` | After owner-confirmed permission, all 20 cases enumerated: 4 raw passes, 16 failures |
| 15 | integrated `1fc0174` | All 20 enumerated and executed in about 15 minutes: 11 raw passes, 9 failures |
| 16 | integrated `dd3efaf` | All 20 enumerated and executed in 1,007 seconds: 17 raw passes, 3 failures |
| 17 | integrated `1a5b648` | Preparation blocked by nested SwiftPM sandbox; no UI cases executed |
| 18 | integrated `1a5b648` | Host build completed; XCTest timed out enabling automation mode before executable discovery; no cases executed |

The reporter conservatively invalidates batch acceptance when xcodebuild fails. Thus
attempt 15 contains 9 failed and 11 blocked normalized results; **the suite did not pass**.
Individual raw results remain in `xcresult-tests.json`, with logs and the full xcresult.
No failures, thresholds or coverage gaps were removed to obtain a passing report.
Attempt 16 similarly normalizes to 3 failed and 17 blocked results. Attempts 17–18
were scoped to the three repaired cases but never reached execution. Neither earlier
raw passes nor final compilation establish a passing final native suite.

Attempt 14 artifacts remain in the macOS worktree at
`/private/tmp/native-sketch-macos-composition/artifacts/evidence/019-native-runtime-attempt-14`.
Attempt 15 is in the primary checkout at `artifacts/evidence/019-native-runtime-attempt-15`.
Its subject SHA-256 is `1490bd8e19bee1d553756dacaacb05848648c2ec6b674de31582856dc5003995`;
test executable SHA-256 is `b1c796d9c213404d0a7bbfb744aede3979c6e2f2dc7e0eb4c9b76286547d679f`.
Both use explicit DEBUG-only storage fault controls; ordinary input remains native.

Attempts 16–18 are preserved under the corresponding primary-checkout evidence folders.
Attempt 16 subject SHA-256 is `a42f1da8717e5e6bbcfec3504cbdd7fc1197e9413aab906f6a983398f5aceaaa`;
test SHA-256 is `aa27c5d777b3a240647881cf97ca65a608025e1d0c1c67ebc686079c00967fe1`.
Final attempt 18 subject SHA-256 is `ad65538fb9fdd993fc1a9a8fb7fdb3fac05173b68bc6c639fc8be15f1f8a2a34`;
test SHA-256 is `dea2ea697179cea16782afc20589871b5039f424c69f0d0a7ee595e3497a1abf`.
Its `blocked.json`, `identity.json` and `enumeration.log` retain the initialization failure;
empty discovery/results deliberately claim no executed cases. No system settings changed.

## Confirmed findings and repairs

- Attempt 14's 15 new fixture cases failed before interaction because XCTest could not
  create directories in the evidence folder. `8b73192` uses its own sandbox-writable
  temporary directory and attaches the path. Attempt 15 confirms the subject can write
  real files and worker markers there. No entitlements or system permissions were relaxed.
- Ordinary Quit closed the window but left the app running. The designer independently
  reproduced this after saving, with an adopted filename and idle/clean state. The
  retirement timer previously ran only in the default loop mode. `8b73192` also registers
  its one-shot timer in common/modal modes. An independent reviewer compiled a repaired
  headless modal-retirement pass and a failing old-timer negative control. Attempt 15's
  `EditorJourneyTests/testDrawSaveReopen` passes actual Quit and relaunch.
- Fresh recovery/diagnostic roots failed when their parent did not exist. The Rust owner
  reproduced Access/Access with no parent created at `1fc0174`. Integrated `9ce075a`
  (worker `729fc5c`) provisions missing private ancestors on the worker with mode 0700,
  retains symlink/non-directory/permissive-service-root rejection, and never chmods existing
  directories. Four targeted tests, full core gates, all-feature service tests and strict
  Clippy pass. Independent review reran all four tests with no actionable findings.
  Attempt 16 confirms native fresh-root creation and recovery journeys; the discard
  confirmation test's ambiguous control query was subsequently repaired separately.
- Attempt 15 also exposed pending-Quit cancellation and deferred-error presentation
  problems. AppKit disables menus during deferred termination, so cancellation needs a
  native waiting sheet. The designer approved a correlated `AWAIT_WRITES` effect with
  Cancel/Return/Escape, while ordinary background saves stay nonmodal. `837286d` adds the
  correlated waiting effect; `dd3efaf` presents deferred failures after actual sheet detachment.
- Three failure-path tests used a generic first-sheet query that rebound to the expected
  successor error sheet. The routing test also attempted a toolbar selection while a
  transient popover consumed the click. These test-oracle repairs must retain the error
  assertions and explicitly verify tool selection before drawing.

## Final repaired candidate: `1a5b648`

Attempt 16 reduced the failures to three concrete cases:

- AppKit's default-button assignment replaces the Cancel button's Escape key equivalent
  with Return. The designer independently reproduced the Escape failure. The final native
  `NativeWaitingPanel` separates Return's default button from responder-chain `cancelOperation`;
  both finish the same sheet and retain the Rust continuation/token guard. Its visible heading,
  content-fit layout and initial Cancel focus follow designer review. The expanded test checks
  Return, Escape and mouse cancellation before save/drain/reopen, and retains a screenshot.
- Recovery's destructive confirmation shared its visible title with another control. A unique
  accessibility identifier now selects exactly that confirmation; the non-destructive Return
  default and selected-copy-only assertions remain.
- The deferred-error test incorrectly required global storage idle while Recovery deliberately
  retained its inventory event. Read-only DEBUG observations now expose authoritative Rust save
  counters. The test verifies save admission, acknowledgement and absent destination while the
  recovery sheet remains open, then requires the queued failure after dismissal. Global idle
  semantics are unchanged.

These fixes are integrated and independently reviewed. Their focused XCTest execution remains
pending after attempt 18's host automation initialization failure. A full final-revision suite
also remains pending; do not combine passes from different revisions into a 20/20 claim.

## Background verification and review

`1fc0174` core checks passed 59 tooling and 144 workspace Rust tests. The integrated
`9ce075a` full background check passes, including fresh-root regressions, real C consumer,
default/feature symbol separation, Swift build/link and headless integration. Two ignored
subprocess entrypoints remain exercised by their parent tests. Expected corrupt-input
failure output in the headless journey is not a failed check.

Logs: `artifacts/verification/019-integrated-runtime/check-9ce075a.log` and
`final-1fc0174-core.log`. The revision-bound coverage run importing attempt 15 records
41 passing rows out of 213, with native failures retained. Its immutable snapshot is
under `artifacts/evidence/history/`; coverage-report generation is not full acceptance.

At final source `1a5b648d26bcdf4c5bb4577c4d5620b749e6856c`, the full background integration
check passes: 59 tooling tests, 148 workspace Rust tests, format/lint/type/boundary gates,
real C consumer, explicit test-control symbol separation, Swift build and headless smoke.
The ordinary feature-disabled app was rebuilt at `artifacts/NativeSketch.app`, executable
SHA-256 `35344f538860afe3282a9225e0c3387bbe4452e1442d9b2a39c5b53488b34cbf`.
Log: `artifacts/verification/019-integrated-runtime/check-1a5b648.log`.
Fresh review of `dd3efaf..1a5b648` found no actionable issues and independently passed Swift 6
DEBUG typechecking in an isolated cache. Reviewed files satisfy the 200-line limit; review
does not certify native event delivery or rendered layout.

The final revision-bound coverage report records 41 passing rows out of 213 and exits nonzero
for required native IDs without current executable discovery. This is an explicit incomplete
acceptance report, not a failed Rust test or a native pass. Its log is
`artifacts/verification/019-integrated-runtime/evidence-attempt18-1a5b648.log`;
the immutable snapshot retains the final source identity and empty native envelopes.

Designer inspection of frozen attempt 14 verified native Save As cancellation and readable
empty recovery presentation; it reproduced the Quit hang. Originating-field restoration,
populated recovery, narrow/dark layout, full color-profile popup, VoiceOver speech and
physical pen behavior are not certified. Shift-Tab traversal and the XCTest Disabled
attribute discrepancy remain unresolved. See `quality/features/native-command-adapter.json`
for explicit uncovered subcases, independently reviewed alongside the executable mappings.

The production painter remains provisional Core Graphics until task 020. No renderer
performance claim, other-platform acceptance, main merge or release is implied.

Final-candidate direct inspection was also blocked: the designer's CUA app-selection call
started around 20:08 UTC and remained pending at the 20:10 cutoff, returning no UI state.
It was interrupted; no keys, clicks or screenshot were obtained. Final panel appearance
and actual Return/Escape/click behavior therefore remain unverified. The worker released
the disposable save gate at 20:09:30. After inspection ended, the lead identified and
terminated only the isolated attempt18 Subject process (PID 19399); this cleanup is not
evidence of normal application Quit. The ordinary app remains built, not launched for handoff.
