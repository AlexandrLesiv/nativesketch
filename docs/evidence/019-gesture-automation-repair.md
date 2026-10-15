# Task 019: maintained gesture automation

## Current checkpoint: both maintained gesture cases pass

Attempt 32 executed both cases on clean `5933c5d5eabb776cf3bb7dc41f6fcc827cab937d`
after the owner reserved 22:36:34–22:41:34 UTC on October 15 (01:36:34–01:41:34 Kyiv
on October 16). Both passed, with xcodebuild exit 0: deactivation in 52.949 seconds and
resize in 49.795 seconds. The complete target enumerated 45 cases; only these two ran.
The lead confirmed that the owned Subject, focus helper and stable runner had exited and
released the desktop early, around 22:39 UTC. No further foreground work is authorized.

Both cases observe Interaction=2 while the pointer is held, then Interaction=0 before
mouse-up. Resize changes the actual window from 1000×780 to 960×750. Deactivation now
launches the paired disposable app, verifies its actual foreground identity and observes
`subject.isActive=false`. Each case then checks unchanged saved bytes, successful new
drawing, two Undo operations and Redo restoring the original bytes. The final document
is clean with one stroke. These are synthetic native input checks, not physical pen tests.

Evidence: `artifacts/evidence/019-native-runtime-attempt-32/`, including raw xcresult,
selected/discovered IDs, normalized results, exported attachments, both preserved fixtures,
`gesture-observations.json` and `focus-helper-validation.json`. Runtime provenance passes
for both cases; host and loaded external test identities are verified after execution.

- Subject SHA-256: `9b4587d779b1f66adb66545b8062db20e8b1f19db1cfd31a23af3269c67c7f34`.
- Test SHA-256: `e797a2501171ff6a39194b7f329e222b8619dd28bcc46b8834d53d966d53165c`.
- Focus helper SHA-256: `1e8bc295a49c39cafd86d6187e1faab1166c4ac3efef61faec8697bf19494a81`.
- Stable host executable SHA-256: `8a7f4a10579762169dc27b4c7896555df84d14fc56ec471a5d4ffd14b00679eb`.

The revision-bound report imports this run successfully. It reports zero complete acceptance
rows out of 213 because these two cases alone do not satisfy any entire mapped requirement;
no core tests or other native cases were executed by that reporting command. The two native
passes remain explicit in the raw and normalized evidence. Log:
`artifacts/verification/019-integrated-runtime/evidence-attempt32-5933c5d.log`.
Earlier failures remain unchanged below. Physical pen behavior, VoiceOver speech, Shift-Tab,
other-platform acceptance and owner acceptance of task dependencies remain separate.

Fresh independent review read the actual xcresult and attachments, rehashed all three
executables, checked actual focus-helper ownership/foreground PID and both preserved
one-stroke files, and verified the executed cancellation/history assertions and teardown.
No actionable finding remained. The broader focus acceptance row also maps nine other
tests: this run does not rerun or certify those cases. Earlier designer acceptance remains
scoped to the unchanged product interface; this repair changed test infrastructure only.

The ordinary app was rebuilt without test controls after execution, without launching or
replacing the existing running instance. Artifact: `artifacts/NativeSketch.app`; executable
SHA-256: `6236ff6837c5753b67e3380e8964c76621ba43a53a42f91e08740f08f577e417`.
Log: `artifacts/verification/019-integrated-runtime/ordinary-build-5933c5d.log`.
The clean full integration check at `3f1f63c` covers the same implementation; `5933c5d`
only adds documentation. No main merge, publication or owner acceptance is implied.

## Earlier checkpoint: resize verified, deactivation driver pending

The owner approved a new desktop session from 22:06:53 to 22:26:53 UTC on October 15
(01:06:53–01:26:53 Kyiv on October 16). No foreground work continues after this cutoff.
Attempts 26–31 retain separate immutable evidence under
`artifacts/evidence/019-native-runtime-attempt-N/`; all discovered 45 cases.

| Attempt | Clean source | Selected outcome |
| --- | --- | --- |
| 26 | `b1e972e` | Both failed Accessibility preflight; actual external test bundle loading verified |
| 27 | `b1e972e` | Both still failed preflight after the owner enabled the visible runner entry |
| 28 | `b1e972e` | Permission passed after owner removed/re-added the stale entry; both failed to observe held input from direct PID events |
| 29 | `1fba3fb` | Public exact-window event metadata did not fix direct PID delivery; both still failed before active input |
| 30 | `7e3608d8e25df11fab0aeaf006d7ebf0c9f4b943` | Resize case passed, 50.874 seconds; validated runtime provenance |
| 31 | `7e3608d8e25df11fab0aeaf006d7ebf0c9f4b943` | Deactivation case failed, 47.300 seconds: held input observed, but the runner's helper window did not deactivate the Subject |

Detailed OS diagnostics proved the stale permission cause: Accessibility requests from
the exact frozen host were checked against an obsolete code-signing requirement and
returned `-67050`, `errSecCSReqFailed` in the installed Security framework header.
The owner manually removed and re-added the stable runner. Automation did not edit the
permission database, change entitlements, or grant access. Scoped private logs are retained
as `tcc-attempt27-detail.log` under `artifacts/verification/019-integrated-runtime/`.

The final input repair uses the public session event tap, after verifying the owned app
is frontmost and both planned points hit its canvas through system AX hit-testing. Cleanup
always balances held input even after intentional deactivation. There is no silent route
fallback, editor command injection, or relaxed cancellation assertion. The ineffective
PID routing attempts remain visible in history.

Attempt 30 proves Interaction=2 before a real 1000×780 → 960×750 resize, Interaction=0
before mouse-up, unchanged saved bytes, successful fresh drawing, two Undo operations and
Redo restoring the original bytes. The final document is clean with one stroke. Fresh
independent review checked the raw outcomes, attachments, hashes and executed assertions.
Attempt 31 stops at the actual-inactivity prerequisite; it does not establish a product
deactivation bug or a cancellation pass. Its framework cleanup ran, and its runner/Subject
had exited when checked at the end of the session. No process was forcibly terminated.

Both final attempts used app SHA-256
`a19b3a4bd5c67af32c22e0bb1516ec6bfc1a6ab84c788c85758035653836ca62`
and external test SHA-256
`b1cecf554ed6fa180fb24946ba781510920805c1d01a38b0c9b44fe839c06def`.
The frozen host identity remained identical across external test rebuilds in attempts
28–31, with successful permission checks. This verifies the scoped rebuild behavior;
future toolchain or OS upgrades remain separate. All modified Swift helpers build and
meet the 200-line limit; fresh independent review found no source blocker. Three focused
Node selection/evidence-route tests pass. Earlier full background checks are listed below.

### Prepared deactivation-driver repair

Clean `3f1f63c76932a401b50842a327682db3e318e5c8` replaces runner-window activation
with an isolated `FocusSubject.app` copied from the same test build. XCTest launches the
paired app using its own disposable runtime root. The helper verifies its exact sibling
path, paired bundle ID, executable SHA and actual foreground process; it refuses to launch
over any pre-existing same-ID process. Cleanup only terminates a helper this test attempted
to launch, releases held input first, and remains registered before helper launch. Actual
Subject inactivity, cancellation, saved-byte and history assertions are unchanged.

Full background integration passes at this revision: 70 tooling tests, the Rust workspace
and feature checks, current format/type/lint/source/dependency gates, C consumer, symbol
separation, Swift build and headless integration. Log: `check-3f1f63c.log` in the integrated
verification directory. Fresh independent review of worker commits `93e2f83` and `c139a5d`
closed a pre-existing-helper ownership finding and reported no remaining actionable issue;
it independently rebuilt the Swift test target and checked helper identity/copy regression.

Primary preparation passed app/helper/test builds, real XCTest cleanup and oracle regressions,
then deliberately exited 1 at the foreground guard. No enumeration or UI ran. Evidence is
`artifacts/evidence/019-focus-helper-preparation-3f1f63c/`:

- Subject SHA-256: `a1997f44d4f426d5bc48e348b1143062017a95a01687391e811e39113fb36cb4`.
- External test SHA-256: `e797a2501171ff6a39194b7f329e222b8619dd28bcc46b8834d53d966d53165c`.
- Focus helper SHA-256: `8c2c087cf712384274f98e45b569c72918c09c29750ad08f5360e3a6a8d5918b`.

The frozen host identity is unchanged. The ordinary app was rebuilt afterward without
launching it; preparation was not a native pass. Attempt 32 above subsequently verified
actual focus transfer with this helper. Physical pen behavior and broader accessibility
gaps remain separate. Task 019 is not marked done without owner acceptance.

## Earlier preparation and failures

Prepared source: `8d3efe6a16295242a25ab035a36ead93384e1d5d`, clean task branch.
This records background preparation, not native execution or task acceptance.
The owner requires a new desktop reservation before any foreground checks.

## Diagnosis and repair

The earlier standalone probe failed before drawing: Open completion timed out, one retry
stopped at its foreground precondition, and another timed out locating a focused Go To
field. Its retained AX tree showed an Untitled document with zero strokes and an Open
sheet. These observations do not establish an OS keyboard-routing defect or a canvas
cancellation bug. The original traces remain unchanged in `artifacts/verification/019-gesture-check/`.

Two maintained `NativeGestureCancellationTests` now create a drawing and save their own
UUID fixture using the existing XCTest native Save helper. There is no separate file-panel
keyboard implementation. A narrow PID-bound CGEvent helper holds the pointer while AX
resizes the real window or a test-owned window deactivates the subject. Both cases require:

- Exact disposable Subject bundle/path identity and existing AX/event-posting permission.
- A strictly positive observed interaction before actual size change or inactivity.
- Cancellation before mouse-up, unchanged saved bytes, and successful subsequent drawing.
- One Undo restoring the original document, a second Undo removing its original stroke,
  and Redo restoring exact original bytes, excluding a hidden cancellation history entry.
- Cleanup of held input, window size, owned window and the runner's activation policy.

No production app or Rust interface changed. These are synthetic native input tests,
not physical pen measurements. Missing permissions fail explicitly; tests never request
permissions or change system settings.

The Node runner accepts repeated exact `--only` IDs, enumerates the whole target before
selection, rejects unknown/duplicate selections and retains selected-only outcomes. It
does not turn unselected cases into passes. The gesture input route is identified separately.

## Background verification and review

At the prepared revision, `node scripts/check.ts` passed: 62 tooling tests, 151 workspace
Rust tests, format/lint/type/source/dependency gates, C consumer, test-symbol separation,
Swift build and headless integration. Log: `artifacts/verification/019-integrated-runtime/check-8d3efe6.log`.
The scoped runner has 11 passing option/selection/Xcode-normalization checks.

Native build-for-testing passed in the implementation worktree for arm64/x86_64 and in
independent review for arm64. Review found a runner activation-policy cleanup defect;
the repair restores the previous policy unconditionally and passed a fresh independent
build. Final review reported no remaining actionable findings in this scope. The new
Swift files are 94, 151 and 44 physical lines.

Primary-checkout preparation completed app/test builds and oracle regressions, then exited
nonzero deliberately at the foreground guard. No native executable enumeration, permission
preflight or UI case ran. Evidence directory:
`artifacts/evidence/019-gesture-repair-preparation-8d3efe6/`.
Subject SHA-256: `c4a2a02ffb7f7aa23617239678b974e0aa1a65004bd67d63c46274a19ad32d03`.
Test SHA-256: `cc20d121ca09eaa06416de43a82e5a7848c10fa4b4d042a149f657db101b1c33`.

## Pending native acceptance

### Reserved execution and follow-up diagnosis

The owner reserved 21:33:30–21:48:30 UTC on October 15. The desktop was released
at 21:47:37. Attempts 23–25 each discovered 45 tests and selected the two gesture
cases. **Neither held gesture began; all six selected outcomes failed.**

| Attempt | Clean source | Observed stopping point |
| --- | --- | --- |
| 23 | `5d17d45ec4c328b3cfee0377be690861441d683e` | Native drawing/Save setup passed; runner Accessibility preflight failed |
| 24 | `5d17d45ec4c328b3cfee0377be690861441d683e` | Accessibility preflight passed; fixture URL equality rejected identical paths with different directory flags |
| 25 | `a6a5caf85d1526e07d926f0262fe8c3359365b21` | Rebuilt tests again failed Accessibility preflight after drawing/Save setup |

Each attempt retains `identity.json`, discovery, selected IDs, normalized/raw results,
logs, attachments and fixtures under `artifacts/evidence/019-native-runtime-attempt-N/`.
Attempt 24 also exposed an automation cleanup defect: an XCTest assertion abort bypassed
Swift `defer`, leaving one disposable Subject running. `198e103` compares canonical
paths; `a6a5caf` registers framework teardown and removes assertions from pointer cleanup.
Attempt 25 confirms fixture teardown after a thrown permission error, not cleanup of an
actually held gesture. The attempt 24 Subject was terminated by exact identity; the
existing ordinary app instance was preserved. See attempt 25 `cleanup.json`.

At the owner's request System Settings was opened. This macOS version labels the pane
“Device Control and Data Access”; the runner switch was visibly on. No permission was
changed by automation. A background experiment at `a6a5caf` showed that changing a nested
test resource and re-signing the ad-hoc host changes its executable hash and designated
requirement. Changing an external test bundle leaves the host identity unchanged. This
supports the identity diagnosis but does not prove the OS's permission decision.
Raw experiment: `artifacts/verification/019-runner-identity/`.

The follow-up repair is integrated at clean `07b895b701219084d623d4cdb3fca2e726282471`.
It separates a stable host from the current test bundle, verifies loaded-bundle provenance,
and expands framework-owned cleanup to the other vulnerable fixtures. Each passing case
must supply matching runtime provenance in exported XCTest attachments. Missing, stale or
mismatched provenance prevents normalized passing results, including for old tests without
the new Swift guard. Host or test-bundle mutation also prevents acceptance.

At that revision the full background integration check passed: 69 tooling tests, 151
workspace Rust tests plus separately exercised feature tests, format/type/lint/source and
dependency gates, real C consumer, test-symbol separation, Swift build and headless smoke.
The first attempt reached a Swift manifest sandbox restriction; the rerun with normal
host build access passed. Both logs are retained as `check-07b895b.log` and
`check-07b895b-unsandboxed.log` under `artifacts/verification/019-integrated-runtime/`.

Primary background preparation then passed app/test builds, oracle checks and the real
Apple XCTest negative cleanup regression, and deliberately stopped at the foreground guard
with exit 1. Its exact trace proves cleanup after an assertion abort and remaining LIFO
cleanup after a throwing block. No native enumeration or application launch occurred.
Evidence: `artifacts/evidence/019-stable-runner-preparation-07b895b/`.
Subject SHA-256: `37b7252dbff5717a1db8ab62a0b0c3ec2f64b58be68f25c6b59e9bb3763b058c`.
External test SHA-256: `c0aec3a3d748746eb2df6419a8d33dc1e3fb80289e46cfcb865ef33f5cae0a50`.
Stable host executable SHA-256: `8a7f4a10579762169dc27b4c7896555df84d14fc56ec471a5d4ffd14b00679eb`.
The ordinary app was rebuilt without test controls after preparation, without launching
or replacing the existing running instance. Its on-disk executable SHA-256 is
`f76b3750a2d474e619a37d928b6719eeb3868769bd41f330d8f2e02c907c8445`;
the test-control symbol scan is empty. Log: `ordinary-build-07b895b.log`.

Fresh independent review of worker commits `a04e821` and `d2b4bb2` found no remaining
actionable issue. It independently ran the seven new Node regressions and actual Apple
XCTest cleanup regression and cross-checked signed bundle identities. All changed sources
meet the 200-line cap. Successful builds, review and signature checks cannot establish
external bundle loading, permission continuity or native gesture behavior.

After explicit desktop approval, enumerate and execute:

- `NativeGestureCancellationTests/testResizeCancelsHeldStrokeAndPreservesSavedDocument`
- `NativeGestureCancellationTests/testDeactivationCancelsHeldStrokeAndPreservesSavedDocument`

Use the stable host under `artifacts/e2e-stable-runner/host-v1/`. The owner may need to
authorize this new path once; automation must not change permissions. Require actual
matching runtime provenance, then verify a changed external test bundle can execute with
the unchanged host and retained permission. Stop and diagnose if Xcode cannot load the
external bundle; do not silently fall back to the embedded tests or weaken the guards.

Retain failures, exact source/binary identities and attachments; verify actual runner
permissions rather than inferring them from entitlements. Existing gesture coverage stays
open until execution succeeds. VoiceOver speech, physical pen and broader platform limits
remain as recorded in [desktop validation](019-native-desktop-validation.md).
