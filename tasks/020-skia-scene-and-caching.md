# 020: Integrate Skia scenes and bounded caches

Status: waiting.
Local migration implementation and background results:
[macOS Skia evidence](../docs/evidence/020-skia-macos.md). Native acceptance and release
GPU measurements of the final timestamp/retry repair remain pending: a system dialog
blocks the canvas and computer control refuses access. The desktop is reserved, but the
owner requested no further app permission prompts while absent. Earlier native/designer
results and invalid benchmark attempts are retained; the status is not a pass.
Prerequisite handoff: [018/021 readiness](../docs/evidence/018-021-pre-skia-readiness.md)
identifies the completed macOS/core preparation and retained platform/acceptance limits.
The owner's request authorizes the next local migration work after technical verification;
the board still requires task 019 owner acceptance before a formal status transition.
Keep the production renderer change separate from the prerequisite verification commits.
Owner role: rendering_performance (lead assigns concrete worker).
Starting revision: integration descendant of `codex/github-pages@712059d5`; freeze exact commit at dispatch.
Owned paths: scoped renderer crate/modules; native surface adapter; render fixtures and benchmark runner.
Dependencies: 019.
Roles: rendering_performance, rust_engine, macos_ui, reviewer.

## Outcome

Selected Skia renderer with correct incremental presentation and measurable scheduling.

## Contracts and acceptance

Read [architecture](../docs/architecture.md), [quality](../docs/quality.md), and the
[refactoring sequence](../docs/refactoring-plan.md). One owner changes shared contracts.

- Define immutable scene updates and one GPU-context owner; keep caches out of document.
- Implement geometry reuse/culling, committed background plus live/selection overlay; atomic commit transition without flash/double ink.
- Implement generation-tagged bounded invalidation/eviction/context recovery and uncached reference; defer tiles until measured need.
- Compare identical geometry/canvas/input/caching policies in release runs; record queue/sample age, tails/memory and no physical-latency claim.

## Required evidence

Name affected acceptance/invariant IDs, executable test mappings, exact source/diff and
binary identity, platform/toolchain/device, commands, results and artifacts. Run applicable
current integration checks until Node parity is established; then the verified replacements.
Failures, skips, flakes and unsupported hardware remain visible. Read-only review is not
runtime evidence. Writers use isolated worktrees and do not revert other workers' changes.
No recursive delegation; at most two writers/three subagents and one feature in flight.

## Completion boundary

Production macOS renderer slice with visual equivalence and measured effects. No Vello production framework or imported comparison branch.

Fresh review follows substantive implementation and material fixes. Task status becomes
review when evidence is ready; only owner acceptance permits done. No main merge or release.

