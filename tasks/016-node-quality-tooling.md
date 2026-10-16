# 016: Node automation and code quality gates

Status: waiting.
Implementation: authorized local preparation after task 015 technical verification/review;
acceptance remains waiting on 015. Node orchestration, source/dependency gates and macOS/web
parity are implemented and reviewed; other host parity and language adoption remain pending.
Evidence: [verification and remaining gaps](../docs/evidence/016-node-tooling.md).
No owner acceptance is inferred.
Owner role: build_reliability (lead assigns concrete worker).
Starting revision: integration descendant of `codex/github-pages@712059d5`; freeze exact commit at dispatch.
Owned paths: scripts/; root package/tool config; apps/web build config; CI; command references.
Dependencies: 015.
Roles: build_reliability, reviewer.

## Outcome

One cross-platform Node orchestration path and pinned, executable language quality checks.

## Contracts and acceptance

Read [architecture](../docs/architecture.md), [quality](../docs/quality.md), and the
[refactoring sequence](../docs/refactoring-plan.md). One owner changes shared contracts.

- Replace owned shell/PowerShell and routine Python orchestration with <=200-line .ts modules and explicit process arguments.
- Preserve build/check/run semantics, errors, timeouts, cleanup and paths with spaces/non-ASCII; prove on relevant native hosts before removal.
- Pin compatible tools; configure Oxlint/Oxfmt/Vite, language checks, source-size and dependency gates with negative fixtures.
- Update CI/docs/agents together; retain specialized analysis only with explicit reason and equivalence plan, never import comparison-results tooling.

## Required evidence

Name affected acceptance/invariant IDs, executable test mappings, exact source/diff and
binary identity, platform/toolchain/device, commands, results and artifacts. Run applicable
current integration checks until Node parity is established; then the verified replacements.
Failures, skips, flakes and unsupported hardware remain visible. Read-only review is not
runtime evidence. Writers use isolated worktrees and do not revert other workers' changes.
No recursive delegation; at most two writers/three subagents and one feature in flight.

## Completion boundary

Old/new parity evidence and enabled gates. Missing Windows/other hosts are pending evidence, not permission to delete unverified routes.

Fresh review follows substantive implementation and material fixes. Task status becomes
review when evidence is ready; only owner acceptance permits done. No main merge or release.

