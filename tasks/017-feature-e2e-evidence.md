# 017: Feature registry and native E2E evidence foundation

Status: waiting.
Local preparation: registry/schema, Cargo/Node discovery and HTML/JSON reporting implemented;
macOS XCTest driver builds, but native execution is blocked before discovery by console
ownership. Task completion and upstream owner acceptance remain pending. See
[runner](../docs/quality/evidence-runner.md) and
[native evidence](../docs/evidence/017-macos-e2e-foundation.md).
Owner role: build_reliability (lead assigns concrete worker).
Starting revision: integration descendant of `codex/github-pages@712059d5`; freeze exact commit at dispatch.
Owned paths: quality/schema/; quality/features/; scripts test/result modules; scoped platform test targets.
Dependencies: 016.
Roles: build_reliability, rust_engine, macos_ui, windows_ui, reviewer.

## Outcome

Discover which acceptance cases have executable tests and report actual per-platform results.

## Contracts and acceptance

Read [architecture](../docs/architecture.md), [quality](../docs/quality.md), and the
[refactoring sequence](../docs/refactoring-plan.md). One owner changes shared contracts.

- Implement schema and runner discovery adapters; join stable IDs to exact revision/binary/platform results.
- Reject missing/duplicate/stale/mismatched tests; preserve skipped/blocked/flaky outcomes and deliberate gap records.
- Generate HTML/JSON feature-platform matrix with logs/screenshots/traces; do not use code coverage as E2E proof.
- Prove macOS built-app launch/control/draw/save/reopen driver; define other platform adapters and explicit visionOS limitation without fake passes.

## Required evidence

Name affected acceptance/invariant IDs, executable test mappings, exact source/diff and
binary identity, platform/toolchain/device, commands, results and artifacts. Run applicable
current integration checks until Node parity is established; then the verified replacements.
Failures, skips, flakes and unsupported hardware remain visible. Read-only review is not
runtime evidence. Writers use isolated worktrees and do not revert other workers' changes.
No recursive delegation; at most two writers/three subagents and one feature in flight.

## Completion boundary

Working registry/report with failure fixtures and initial actual native E2E; remaining platform drivers assigned to port tasks.

Fresh review follows substantive implementation and material fixes. Task status becomes
review when evidence is ready; only owner acceptance permits done. No main merge or release.

