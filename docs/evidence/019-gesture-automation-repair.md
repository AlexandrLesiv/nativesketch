# Task 019: maintained gesture automation

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

The follow-up repair separates a stable host from the current test bundle, verifies
loaded-bundle provenance, and expands framework-owned cleanup to the other vulnerable
test fixtures. Background preparation/review and native validation are separate gates;
successful builds or signature checks cannot establish permission continuity.

After explicit desktop approval, enumerate and execute:

- `NativeGestureCancellationTests/testResizeCancelsHeldStrokeAndPreservesSavedDocument`
- `NativeGestureCancellationTests/testDeactivationCancelsHeldStrokeAndPreservesSavedDocument`

Retain failures, exact source/binary identities and attachments; verify actual runner
permissions rather than inferring them from entitlements. Existing gesture coverage stays
open until execution succeeds. VoiceOver speech, physical pen and broader platform limits
remain as recorded in [desktop validation](019-native-desktop-validation.md).
