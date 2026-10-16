# 025: Project development workbench and Zed integration

Status: waiting.
Owner role: build_reliability (lead assigns scoped workers).
Dependencies: 024.
Roles: build_reliability, ui_ux_designer, reviewer.

## Outcome

A project-specific GUI for docs, tasks/dependencies, feature verification, build/run jobs,
Wasm preview and qualified metrics, integrated with Zed through supported project tasks.
Read [the detailed plan](../docs/development-workbench-plan.md) before implementation.
Implementation and exact background checks are recorded in the
[workbench evidence](../docs/evidence/025-project-workbench.md). Nested-process cancellation
is repaired and independently reviewed; native Zed/speech checks remain explicit gaps.
Available background implementation is ready for review, not owner acceptance.

## Boundaries

- Reuse authoritative repository/evidence data and existing Node commands.
- Share versioned snapshot/view/style modules between local GUI and offline report export.
- Do not implement CI, publish, merge, change security settings or add drawing-app features.
- Zed currently has no documented general custom-panel extension API; do not claim one.
- Keep every handwritten source/test/style <=200 physical lines after formatting.
- Preserve automation existence, exact execution and owner acceptance as separate concepts.
- No arbitrary command execution; allowlisted jobs, bounded resources, loopback security,
  safe document rendering and isolated Wasm preview require tests.

## Evidence and acceptance

Register the 14 DX acceptance IDs in the [designer specification](../docs/design/development-workbench.md). Execute Node
failure/ownership tests, real browser E2E and offline export checks. Designer reviews the
visible GUI; a fresh reviewer assesses command ownership, local service access and evidence.
Actual Zed interaction may remain pending when per-app permission would be required.
Source/build identity, failed attempts, unsupported hosts and remaining gaps stay visible.

The owner's authorization permits downstream local work after the available architecture
review packet while required external-host gaps remain explicit. It does not mark task024
accepted or relax the board's owner-acceptance prerequisite. No main merge or release.
