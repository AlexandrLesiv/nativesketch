# 018: Extract Rust editor responsibilities and typed facades

Status: waiting.
Owner role: rust_engine (lead assigns concrete worker).
Starting revision: integration descendant of `codex/github-pages@712059d5`; freeze exact commit at dispatch.
Owned paths: crates/sketch-core/; crates/sketch-ffi/; crates/sketch-wasm/; narrowly coordinated consumers.
Dependencies: 015, 017.
Roles: rust_engine, build_reliability, reviewer.

## Outcome

Small cohesive shared modules with unchanged editing/persistence semantics.

## Contracts and acceptance

Read [architecture](../docs/architecture.md), [quality](../docs/quality.md), and the
[refactoring sequence](../docs/refactoring-plan.md). One owner changes shared contracts.

- Extract model, interaction, commands, history, geometry, scene, codec and source policy; <=200 physical lines including tests.
- Preserve ABI 2/Wasm 1/bytes; introduce typed internal/native facades without silently changing public layouts.
- Enforce single mutation owner, whole-batch validation, prediction separation, active-interaction state and read-only capability.
- Run invariant/property/real-consumer conformance; isolate unsafe code and prevent panics crossing FFI.

## Required evidence

Name affected acceptance/invariant IDs, executable test mappings, exact source/diff and
binary identity, platform/toolchain/device, commands, results and artifacts. Run applicable
current integration checks until Node parity is established; then the verified replacements.
Failures, skips, flakes and unsupported hardware remain visible. Read-only review is not
runtime evidence. Writers use isolated worktrees and do not revert other workers' changes.
No recursive delegation; at most two writers/three subagents and one feature in flight.

## Completion boundary

Compatibility-preserving extraction integrated with all affected consumers; behavioral differences require separate specified changes.

Fresh review follows substantive implementation and material fixes. Task status becomes
review when evidence is ready; only owner acceptance permits done. No main merge or release.

