# 019: Compose native macOS editor and complete brush E2E

Status: waiting.
Local preparation: reviewed native composition and focused keyboard/field journeys are
integrated; [native evidence](../docs/evidence/019-native-composition.md). The portable
ApplicationSession command router is integrated and independently reviewed at `f4524fb`;
[router evidence](../docs/evidence/019a-application-routing.md). New C/runtime and native
adoption (019B–E) are integrated locally and have scoped native verification against the agreed
[native interface v1](../docs/architecture/019bc-native-interface.md), frozen at `106abfe`
and integrated as `c0b63da`. Twelve new native acceptance rows map to discovered executables
in `quality/features/native-command-adapter.json`, with uncovered subcases retained. Twenty
cases passed in attempt 19 at `1672c49` after diagnosing the automation timeout as late
system authentication. The expanded 36-case target at `79f3028` is actually enumerated;
attempt 20 executes and passes the focused Save As field-selection restoration case only.
Attempt 21 passes the other 15 selected additions at `79f3028`. Final candidate `03cad1c`
enumerates 43 cases and passes 13 selected regressions, including the designer-found
inspector/discard repairs, four sheet-focus branches, small/dark layout and genuine
caught-panic Close/Quit. Full background checks pass. These are separate scoped runs;
no all-43-at-final-revision claim is made. Final gesture evidence and visual review are
recorded in the desktop validation report. The board remains waiting on unaccepted
dependencies 018/021; this does not erase the completed native checks.
The remaining held-gesture probe has been replaced by maintained XCTest cases with proven
native Save setup. [Repair preparation](../docs/evidence/019-gesture-automation-repair.md)
records passing builds/review and the explicit new-desktop-approval execution gate.
Subsequent attempts 23–25 discovered 45 cases but both selected gesture tests failed
before held input: permission preflight, then a repaired fixture-path assertion, then
permission preflight after rebuilding. Test-host identity and framework cleanup repairs are
integrated at `07b895b`; full background checks, preparation and fresh independent review
pass. Gesture acceptance remains open; the last desktop reservation has ended. Exact
failures, cleanup and prepared binary identities are retained in the repair report.
The next reserved session verified actual resize cancellation in attempt 30 at `7e3608d`,
including saved-byte preservation, fresh drawing and Undo/Redo. Attempt 31 reached held
input but its helper window failed to deactivate the Subject; this automation step remains
open. Stable runner permission survives the changed external test bundle after the owner
replaced a stale permission entry. The session has ended; no new desktop use is authorized.
A paired disposable focus-helper repair is integrated and prepared at `3f1f63c`, with full
background checks and fresh review passing. Actual focus transfer still requires a new
approved native run; previous resize success does not certify this changed test bundle.
The designer reproduced the original ordinary Quit hang, which attempt 15 confirms repaired;
attempt 16 also verifies fresh recovery-root provisioning. See
[desktop validation](../docs/evidence/019-native-desktop-validation.md)
for exact revisions, reviews and preserved failures. Background integration and Wasm checks
pass; remaining coverage limits are recorded explicitly.
See [runtime evidence](../docs/evidence/019bc-native-runtime.md).
Save As keyboard delivery passes after
the owner disabled a conflicting system shortcut; the original app binding was preserved.
The [agreed staged bridge contract](../docs/architecture/019-session-bridge.md) defines 019A–E.
Owner role: macos_ui (lead assigns concrete worker).
Starting revision: integration descendant of `codex/github-pages@712059d5`; freeze exact commit at dispatch.
Owned paths: apps/macos/; macOS E2E fixtures; only agreed shared contract changes.
Dependencies: 018, 021.
Roles: macos_ui, rust_engine, ui_ux_designer, reviewer.

## Outcome

AppKit window/presenters/input/canvas/document services with clear ownership.

## Contracts and acceptance

Read [architecture](../docs/architecture.md), [quality](../docs/quality.md), and the
[refactoring sequence](../docs/refactoring-plan.md). One owner changes shared contracts.

- Extract window composition, bridge, presenters, brush/menu controls, input adapter, canvas and coordinator into cohesive <=200-line files.
- Canvas no longer reads widgets; shared tool state drives controls and action availability; field editing/shortcuts stay native.
- Preserve cancellation, coalesced samples, missing pressure, selection and keyboard access; instrument operation boundaries.
- Run real UI journeys, AX/keyboard/focus/display/resize checks, designer review; record real pen versus replay and unavailable hardware.

## Findings carried from task 017

- Command-Shift-S failed to expose a Save As panel in native attempt 6, despite a defined
  shortcut/enabled menu. The native File → Save As menu passes. Diagnose responder/menu
  routing and test the shortcut independently with canvas focus; do not infer keyboard
  equivalence from the menu journey. Resolved diagnosis: external system shortcut conflict;
  actual keyboard tests pass at `458f923`. Stronger ordinary-Save change assertions follow
  separately; do not interpret this as a change to the application's shortcut binding.
- The exact-PID AX probe on `79f3028` found canvas AXEnabled unsupported, not false.
  This explains why the XCTest “Disabled” label cannot establish a disabled canvas;
  VoiceOver speech remains separate and unverified.
- On the next relevant UI-test change, capture the inspector after committed width/color
  and capture the expanded native color-profile menu without window-boundary cropping.
  Existing AX and strict saved-color assertions support the current narrow journey.

These findings came from actual task 017 artifacts and the designer review, not a new
requirement to run native UI on every background check.

## Required evidence

Name affected acceptance/invariant IDs, executable test mappings, exact source/diff and
binary identity, platform/toolchain/device, commands, results and artifacts. Run applicable
current integration checks until Node parity is established; then the verified replacements.
Failures, skips, flakes and unsupported hardware remain visible. Read-only review is not
runtime evidence. Writers use isolated worktrees and do not revert other workers' changes.
No recursive delegation; at most two writers/three subagents and one feature in flight.

## Completion boundary

Running macOS slice and actual UI evidence; keep renderer swap separate in 020.

Fresh review follows substantive implementation and material fixes. Task status becomes
review when evidence is ready; only owner acceptance permits done. No main merge or release.

