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

The reporter conservatively invalidates batch acceptance when xcodebuild fails. Thus
attempt 15 contains 9 failed and 11 blocked normalized results; **the suite did not pass**.
Individual raw results remain in `xcresult-tests.json`, with logs and the full xcresult.
No failures, thresholds or coverage gaps were removed to obtain a passing report.

Attempt 14 artifacts remain in the macOS worktree at
`/private/tmp/native-sketch-macos-composition/artifacts/evidence/019-native-runtime-attempt-14`.
Attempt 15 is in the primary checkout at `artifacts/evidence/019-native-runtime-attempt-15`.
Its subject SHA-256 is `1490bd8e19bee1d553756dacaacb05848648c2ec6b674de31582856dc5003995`;
test executable SHA-256 is `b1c796d9c213404d0a7bbfb744aede3979c6e2f2dc7e0eb4c9b76286547d679f`.
Both use explicit DEBUG-only storage fault controls; ordinary input remains native.

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
  Native recovery journeys still require the repaired candidate.
- Attempt 15 also exposed pending-Quit cancellation and deferred-error presentation
  problems. AppKit disables menus during deferred termination, so cancellation needs a
  native waiting sheet. The designer approved a correlated `AWAIT_WRITES` effect with
  Cancel/Return/Escape, while ordinary background saves stay nonmodal. Repair and native
  verification remain pending at this checkpoint.
- Three failure-path tests used a generic first-sheet query that rebound to the expected
  successor error sheet. The routing test also attempted a toolbar selection while a
  transient popover consumed the click. These test-oracle repairs must retain the error
  assertions and explicitly verify tool selection before drawing.

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

Designer inspection of frozen attempt 14 verified native Save As cancellation and readable
empty recovery presentation; it reproduced the Quit hang. Originating-field restoration,
populated recovery, narrow/dark layout, full color-profile popup, VoiceOver speech and
physical pen behavior are not certified. Shift-Tab traversal and the XCTest Disabled
attribute discrepancy remain unresolved. See `quality/features/native-command-adapter.json`
for explicit uncovered subcases, independently reviewed alongside the executable mappings.

The production painter remains provisional Core Graphics until task 020. No renderer
performance claim, other-platform acceptance, main merge or release is implied.
