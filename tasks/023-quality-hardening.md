# 023: Recovery, accessibility and performance hardening

Status: waiting.
Owner role: build_reliability (lead assigns concrete worker).
Starting revision: integration descendant of `codex/github-pages@712059d5`; freeze exact commit at dispatch.
Owned paths: fuzz/property/sanitizer targets; visual/performance fixtures; accessibility checks; coverage matrix.
Dependencies: 022.
Roles: build_reliability, rust_engine, rendering_performance, ui_ux_designer, reviewer.

## Outcome

Additional quality lanes find failures beyond happy-path E2E.

## Contracts and acceptance

Read [architecture](../docs/architecture.md), [quality](../docs/quality.md), and the
[refactoring sequence](../docs/refactoring-plan.md). One owner changes shared contracts.

- Add bounded fuzz targets for codecs/input/commands, property sequences, targeted Miri/sanitizers; Loom only for custom concurrency.
- Exercise cached/uncached rendering, context loss, memory pressure, lifecycle, locale/RTL/large text and screen-reader/keyboard paths.
- Provision explicit per-platform profile lanes: Instruments, Perfetto/Macrobenchmark, WPR/WPA, Sysprof and browser tools.
- Establish noise-aware release baselines on named devices; preserve failure evidence and physical-camera gap. Do not schedule unattended services implicitly.

## Required evidence

Name affected acceptance/invariant IDs, executable test mappings, exact source/diff and
binary identity, platform/toolchain/device, commands, results and artifacts. Run applicable
current integration checks until Node parity is established; then the verified replacements.
Failures, skips, flakes and unsupported hardware remain visible. Read-only review is not
runtime evidence. Writers use isolated worktrees and do not revert other workers' changes.
No recursive delegation; at most two writers/three subagents and one feature in flight.

## Completion boundary

Executed supported lanes with exact environment/binaries and named gaps; no universal performance budget or unsupported sanitizer claims.

Fresh review follows substantive implementation and material fixes. Task status becomes
review when evidence is ready; only owner acceptance permits done. No main merge or release.

