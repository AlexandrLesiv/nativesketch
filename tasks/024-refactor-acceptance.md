# 024: Final architecture and packaged-app acceptance

Status: waiting.
Owner role: build_reliability (lead assigns concrete worker).
Starting revision: integration descendant of `codex/github-pages@712059d5`; freeze exact commit at dispatch.
Owned paths: quality reports; packaging/upgrade tests; final source inventory; change report.
Dependencies: 023, 004, 009, 010, 011, 012, 013.
Roles: build_reliability, ui_ux_designer, reviewer.

## Outcome

A complete review packet for human merge approval.

The [available background packet](../docs/evidence/024-background-acceptance.md) records
executed local checks, exact revisions and remaining native/platform gaps. It is not
final task acceptance; later implementation requires refreshed evidence.

## Contracts and acceptance

Read [architecture](../docs/architecture.md), [quality](../docs/quality.md), and the
[refactoring sequence](../docs/refactoring-plan.md). One owner changes shared contracts.

- Verify zero handwritten source-size violations, dependency/ownership boundaries and removal of unverified legacy automation.
- Run required feature/E2E matrix and package clean-launch/old-document/upgrade tests for selected distribution; retain symbols/build identities.
- Resolve correctness/data-loss findings; record designer and fresh independent review plus remaining device/physical-latency gaps.
- Do not mark all-platform completion when required platforms remain blocked; present narrowed acceptance explicitly if needed.

## Required evidence

Name affected acceptance/invariant IDs, executable test mappings, exact source/diff and
binary identity, platform/toolchain/device, commands, results and artifacts. Run applicable
current integration checks until Node parity is established; then the verified replacements.
Failures, skips, flakes and unsupported hardware remain visible. Read-only review is not
runtime evidence. Writers use isolated worktrees and do not revert other workers' changes.
No recursive delegation; at most two writers/three subagents and one feature in flight.

## Completion boundary

Reviewable artifacts and complete evidence for scoped approval; do not merge main, publish releases or mark owner acceptance on their behalf.

Fresh review follows substantive implementation and material fixes. Task status becomes
review when evidence is ready; only owner acceptance permits done. No main merge or release.

