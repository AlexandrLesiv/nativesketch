# 022: Portable adapter decomposition and conformance

Status: waiting.
Owner role: rust_engine (lead assigns concrete worker).
Starting revision: integration descendant of `codex/github-pages@712059d5`; freeze exact commit at dispatch.
Owned paths: platform bridge contracts/conformance fixtures; targeted apps/ modules; port task evidence.
Dependencies: 020, 021.
Roles: rust_engine, ui_ux_designer, macos_ui, windows_ui, build_reliability, reviewer.

## Outcome

Shared host contracts and scoped extraction plans for every existing platform.

## Contracts and acceptance

Read [architecture](../docs/architecture.md), [quality](../docs/quality.md), and the
[refactoring sequence](../docs/refactoring-plan.md). One owner changes shared contracts.

- Freeze native UI/action/sample/viewer contracts and common byte/command fixtures; retain idiomatic native presenters.
- Split remaining host responsibilities <=200 lines; serialize by host or use isolated worktrees with explicit owners.
- Preserve Apple editor versus viewer capabilities, standalone Watch source policy, Android desktop reuse and semantic HTML.
- Port tasks 004/009-013 own platform E2E/packaging; record blocked drivers/hardware honestly. No invented Android/Linux/web agent config.

## Required evidence

Name affected acceptance/invariant IDs, executable test mappings, exact source/diff and
binary identity, platform/toolchain/device, commands, results and artifacts. Run applicable
current integration checks until Node parity is established; then the verified replacements.
Failures, skips, flakes and unsupported hardware remain visible. Read-only review is not
runtime evidence. Writers use isolated worktrees and do not revert other workers' changes.
No recursive delegation; at most two writers/three subagents and one feature in flight.

## Completion boundary

Conformance boundaries and scoped host extractions ready for individual platform acceptance; not all-platform certification.

Fresh review follows substantive implementation and material fixes. Task status becomes
review when evidence is ready; only owner acceptance permits done. No main merge or release.

