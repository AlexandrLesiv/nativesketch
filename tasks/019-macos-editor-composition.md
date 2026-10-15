# 019: Compose native macOS editor and complete brush E2E

Status: waiting.
Local preparation: independently reviewed mechanical extraction is integrated and tested
at `91efb03`; [evidence](../docs/evidence/019-composition-extraction.md). Typed session/storage
adoption and broader native acceptance remain pending. Save As keyboard delivery passes after
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
- Final AX snapshots report application/window/canvas Disabled with canvas keyboard focus.
  Investigate actual enabled attributes and VoiceOver behavior; the successful mouse journey
  does not certify accessibility or establish the cause of that reporting.
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

