# 015: Architecture contracts and compatibility baseline

Status: review.
Evidence: [implemented baseline and verification](../docs/evidence/015-contracts-and-baseline.md).
Owner role: rust_engine (lead assigns concrete worker).
Starting revision: integration descendant of `codex/github-pages@712059d5`; freeze exact commit at dispatch.
Owned paths: docs/architecture/; quality/features/ initial manifests; scoped compatibility fixtures.
Dependencies: none; selected-base contract work.
Roles: rust_engine, ui_ux_designer, build_reliability, reviewer.

## Outcome

Freeze concrete contracts and an auditable baseline before refactoring.

## Contracts and acceptance

Read [architecture](../docs/architecture.md), [quality](../docs/quality.md), and the
[refactoring sequence](../docs/refactoring-plan.md). One owner changes shared contracts.

- Inventory source files and gate debt at 712059d5; classify handwritten/generated/vendor/data with reproducible counts.
- Define command/query/effect types, sample batching, state ownership, viewport math, scene changes, FFI lifetimes, thread affinity and errors.
- Map existing tests and gaps to invariant/acceptance IDs without inventing passing coverage.
- Capture document bytes, ABI 2 and Wasm protocol 1 fixtures and native interaction differences (resize math/zoom/focus).

## Required evidence

Name affected acceptance/invariant IDs, executable test mappings, exact source/diff and
binary identity, platform/toolchain/device, commands, results and artifacts. Run applicable
current integration checks until Node parity is established; then the verified replacements.
Failures, skips, flakes and unsupported hardware remain visible. Read-only review is not
runtime evidence. Writers use isolated worktrees and do not revert other workers' changes.
No recursive delegation; at most two writers/three subagents and one feature in flight.

## Completion boundary

Reviewed contract/file/type map, baseline inventory and compatibility fixtures; no broad implementation or renderer changes.

Fresh review follows substantive implementation and material fixes. Task status becomes
review when evidence is ready; only owner acceptance permits done. No main merge or release.

