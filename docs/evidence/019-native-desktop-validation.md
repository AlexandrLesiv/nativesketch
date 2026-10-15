# Native desktop validation

Host: Mac14,10 arm64, macOS 27.0.1 (26A434), Xcode 27.0 (27A266a),
Swift 6.4, Rust 1.99.0, Node 24.22.0. These are native mouse/keyboard
journeys, not physical pen measurements. The owner reserved the foreground desktop
and subsequently extended it through 2026-10-15 21:30 UTC (00:30 Kyiv on October 16).

## Gesture continuation after the earlier session

A separate owner-approved 15-minute reservation ran from 21:33:30 to 21:48:30 UTC;
the desktop was released at 21:47:37. Attempts 23–25 discovered 45 cases and each ran
the two selected maintained gesture cases. All stopped before held input. Native file
setup passed, while runner permission and test-harness defects prevented gesture acceptance.
The [gesture repair report](019-gesture-automation-repair.md) records revisions,
diagnosis, repairs and retained evidence. No resize/deactivation cancellation pass is claimed.
Later repairs and checks are background-only until a new reservation is approved.

## October 15 continuation

The earlier failures below remain preserved. The automation timeout was diagnosed before
retrying: attempt 18 requested macOS authentication around 20:06 UTC and timed out around
20:07; the authentication completed at 20:15:25, after that runner had exited. The console
UID matched the execution user. No system security or global trust setting was changed.

Attempt 19, exact source `1672c49a5219c9d1f97bb839a4db101f8ed08412`, enumerated and passed
all 20 native cases in 1,057 seconds, with xcodebuild exit 0. This verifies the three
previously repaired failures, including Return/Escape/click cancellation of pending Quit.
Subject SHA-256: `fbf5ff87a787f5289a473fe3284ff2a5ca4cbb6bcd5e3f3a03a3161cbd1dc4d2`;
test SHA-256: `dea2ea697179cea16782afc20589871b5039f424c69f0d0a7ee595e3497a1abf`.
The revision-bound report records 49 passing rows out of 213; it is not whole-product acceptance.

The acceptance audit then found two implementation gaps. Native file intents discarded the
originating field/selection; a weak responder, session and generation guarded restoration now
preserves native selection without stealing focus from a newer control or inactive window.
Recovery capacity/usage fields were always zero; bounded real inventory accounting now
reports occupied identities and retained bytes, including resolved entries and staging files.
Unknown/incomplete accounting is explicit; the reported capacity flag is not a promise that
an arbitrary next document fits. Tests use actual files and the production worker boundary.

Attempt 20 at `79f3028f7757cdca1b04f37929f95c140e93ac41` enumerated all 36 cases and executed
only `NativeFocusRestorationTests/testCancelledSaveAsRestoresNativeWidthSelection`: **1/1
passed**, xcodebuild exit 0, 37.5 seconds. Width `12` with its final digit selected survives
native Save As cancellation; subsequent typing without clicking produces `18`.
Subject SHA-256: `db0268a33a71e9b93588a100b00fd8e7ab979bfcc094f71eee3ff22029b5319a`;
test SHA-256: `3ca616f74338b5833448c53ebc05bbb30a1e9acb68e307a73e93c73d625c7b18`.
The other 15 new cases were discovered, not executed by this attempt. Earlier revision
passes remain historical; the exact-source report retained 41 passing rows out of 213.

The full background check at `79f3028` passed: 59 tooling tests, 151 workspace Rust tests,
format/lint/type/source/dependency gates, real C consumer, default/feature symbol separation,
Swift build/link and headless smoke. Feature-only read-control tests separately passed 4/4.
Fresh independent reviews reproduced focused Rust and Swift checks and closed the scoped
findings. Source review and headless execution do not establish native interaction success.

Independent designer inspection on the attempt 20 subject confirmed Save As and Recovery
Later preserve the originating width selection, readable inspector and populated recovery,
keyboard selection, safe Escape cancellation, and normal Quit. The exact-PID AX probe found
canvas `AXEnabled` unsupported (`-25205`), not false: XCTest's “Disabled” text is not evidence
that the canvas is disabled. VoiceOver speech remains untested. The waiting-panel screenshot
was clear and unclipped; its full-desktop background makes it private evidence, not a public
product image.

The designer also reproduced inspector Escape losing canvas shortcuts and Return doing
nothing in recovery-discard confirmation. Candidate `9b7e147` integrates guarded inspector
focus restoration and a native confirmation panel with separate safe Return/Escape paths.
Its strengthened test requires actual dismissal, preventing the previous no-op Return from
passing merely because recovery rows remained. An explicit test-only caught-panic hook and
native Close/Quit journey cover engine unavailability; the hook is excluded from ordinary
builds. Additional native tests cover Recovery Later and failure-cancellation field selection.
Fresh code and design reviews found no actionable issue; runtime verification is separate.

Artifacts are under `artifacts/evidence/019-native-runtime-attempt-{19,20}` and
`artifacts/verification/019-integrated-runtime/`. Attempts retain exact identities, discovery,
selected execution, logs, xcresult attachments and disposable fixtures.

Attempt 21 at the same source `79f3028` passed all **15/15** newly added selected cases,
with xcodebuild exit 0 in 1,092 seconds. Subject SHA-256:
`a2087ca9ffbcab28ccf20fcfc0720be4da87e36c7b2ff9b8ee6c857cf2618827`;
test SHA-256: `3ca616f74338b5833448c53ebc05bbb30a1e9acb68e307a73e93c73d625c7b18`.
The app build identity differs from attempt 20; these remain two separately identified runs.
All 15 disposable fixture roots and xcresult attachments are retained in the attempt 21 folder.

Final candidate `03cad1c2d5261dbdd8b441e5c29cb9a91cc1b0f0` additionally includes the
designer repairs, caught-panic native test boundary, direct and successor-sheet focus cases,
small/dark layout evidence capture and uncropped private color-profile capture. Both the
current Node integration entry point and the legacy `./scripts/check` pass. The Node check
includes 59 tooling tests, 151 workspace Rust tests, five separately executed feature tests,
default release test-symbol exclusion, actual C consumer and linked Swift headless checks.
Exact logs: `check-03cad1c.log` and `legacy-check-03cad1c.log` in the verification directory.

Attempt 22 enumerated **43** cases and passed all **13/13** selected affected native
regressions, xcodebuild exit 0, in 584 seconds. Source remained clean and frozen throughout.
Subject SHA-256: `3252a1d4f15bd7613d4e3ceb029fc7773393c427dfdd3c8e3a0f7da3e862f52e`;
test SHA-256: `6628a43c7afbba1c15b38256b5885cd828c90ec8f7be597c516b746593d4338a`.
The selected cases cover the designer's inspector/discard repairs, small/dark layout,
genuine caught-panic Close/Quit, four sheet-focus branches and existing field/editing
regressions. This is not a claim that all 43 executed at this revision. The exact-source
coverage report imports only attempt 22 and records 45 passing rows out of 213, exit 0;
unexecuted and other-platform requirements remain visible. Log:
`artifacts/verification/019-integrated-runtime/evidence-attempt22-03cad1c.log`.

Fresh independent review cross-checked all 13 raw xcresult outcomes, the selection list,
execution log, normalized results and actual executable hashes. All 40 mapped native IDs
exist in the 43-case discovery. No product blocker was found. A P3 reporting defect used
whole-suite elapsed time for each normalized test's duration; the runner now uses the
already parsed per-case duration (zero when no case outcome exists). Historical artifacts
are preserved unchanged: the 584 seconds above is the run duration, not each case's time.
The five focused Xcode-parser tests and tooling type/lint/format gates verify this repair;
it does not change app or test binaries and requires no additional foreground run.

The designer reviewed actual final screenshots offline: the complete native color-profile
popup boundary and scroll indicator are visible (`79E9256D-6079-4FFF-985E-D5C20DCB772E.png`),
the small dark inspector fits (`69C5BA78-511C-4ADD-A1D4-8C366C11BE2D.png`), and discard
confirmation clearly shows the safe blue Cancel default (`82BE7956-FA56-498B-BA1F-16BF0D8E9404.png`).
The inspector is fully visible (`F7DCE89A-7C58-4576-BDDA-F17941DFEBE7.png`). No blocking
layout issue was found. Hidden menu rows and a poison-panel image were not inspected.
These are screenshot findings, separate from test interaction evidence. Full-screen
images include unrelated desktop/chat content and must remain private local artifacts.

### Separate gesture probe and remaining limits

A reviewed native CGEvent/AX helper attempted to verify cancellation during a held stroke
on resize and application deactivation, followed by exact saved-byte and undo checks.
It did not reach drawing input: the first run timed out completing native Open; a second
stopped at the foreground precondition; after explicit activation, a third timed out
finding the native Go To field. The helper's long-path transport was repaired and compiled,
but two actual setup cycles still failed. Further retries stopped. This is an automation
setup gap, not a demonstrated product cancellation failure. All three JSON traces and
private AX diagnostics remain under `artifacts/verification/019-gesture-check/`.
No resize/deactivation gesture pass, physical pen measurement or XCTest equivalence is claimed.

VoiceOver speech, physical pen behavior, Shift-Tab traversal and those held-gesture paths
remain unverified. The complete 43-case suite has not run on the final candidate; the scoped
native batches above and full background checks are the evidence available for review.
Other-platform native acceptance, production Skia migration (task 020), and remaining
refactoring tasks are separate. Task 019's board status stays waiting on owner acceptance
of dependencies 018/021; this report does not silently mark those tasks or task 019 done.

### Running app handoff

Foreground work ended at 21:22 UTC, before the 21:30 reservation cutoff. The disposable
Subject's file panel was cancelled and the Subject terminated normally. No pre-existing
ordinary instance was running. The ordinary app was launched as PID 50505, bundle
`local.native-sketch.dev`, at `artifacts/NativeSketch.app`, source `03cad1c`;
executable SHA-256 `c16fe7889c400bf86db1aa20aa6b06c73dec3861e6b3133f1b463f6df9054768`.
The normal binary contains no native test-control symbols. No user document was opened
or modified. Handoff identity is retained in
`artifacts/verification/019-gesture-check/running-app-handoff.json`.

## Preserved runs

| Attempt | Source | Actual outcome |
| --- | --- | --- |
| 12–13 | worker `df804bd` | XCTest automation initialization timed out before discovery; no cases executed |
| 14 | worker `df804bd` | After owner-confirmed permission, all 20 cases enumerated: 4 raw passes, 16 failures |
| 15 | integrated `1fc0174` | All 20 enumerated and executed in about 15 minutes: 11 raw passes, 9 failures |
| 16 | integrated `dd3efaf` | All 20 enumerated and executed in 1,007 seconds: 17 raw passes, 3 failures |
| 17 | integrated `1a5b648` | Preparation blocked by nested SwiftPM sandbox; no UI cases executed |
| 18 | integrated `1a5b648` | Host build completed; XCTest timed out enabling automation mode before executable discovery; no cases executed |
| 19 | integrated `1672c49` | All 20 discovered and executed cases passed; xcodebuild exit 0 |
| 20 | integrated `79f3028` | All 36 discovered; only the selected Save As focus case executed and passed |
| 21 | integrated `79f3028` | All 15 selected additional cases passed; separate app identity from attempt 20 |
| 22 | integrated `03cad1c` | All 43 discovered; all 13 selected regressions passed |

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

## Earlier repaired candidate: `1a5b648`

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

At the attempt 18 checkpoint, these fixes were integrated and independently reviewed, but
their focused XCTest execution remained pending after initialization failed. Attempt 19
subsequently passed all 20 at `1672c49`; later candidates have scoped runs listed above.
Do not combine passes from different revisions into an all-case final-revision claim.

## Historical background verification and review through attempt 18

The pending/unverified statements in this section describe that earlier cutoff only.
Current outcomes and limitations are recorded in the October 15 continuation above.

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

At the earlier 20:10 cutoff, direct inspection was also blocked: the designer's CUA app-selection call
started around 20:08 UTC and remained pending at the 20:10 cutoff, returning no UI state.
It was interrupted; no keys, clicks or screenshot were obtained. Final panel appearance
and actual Return/Escape/click behavior were then unverified. The worker released
the disposable save gate at 20:09:30. After inspection ended, the lead identified and
terminated only the isolated attempt18 Subject process (PID 19399); this cleanup is not
evidence of normal application Quit. At that checkpoint the ordinary app was built, not launched.
Later native runs and designer review are recorded above; handoff is recorded separately.
