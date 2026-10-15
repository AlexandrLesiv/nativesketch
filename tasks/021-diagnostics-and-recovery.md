# 021: Structured diagnostics and revision-safe recovery

Status: waiting.
Local preparation: [storage/recovery contracts](../docs/architecture/021-storage-contracts.md)
agreed at `62d54e8`; Rust admission/snapshot/acknowledgement implementation is in progress.
Native adoption, diagnostics/recovery execution and upstream acceptance are not yet complete.
Owner role: rust_engine (lead assigns concrete worker).
Starting revision: integration descendant of `codex/github-pages@712059d5`; freeze exact commit at dispatch.
Owned paths: shared diagnostic/storage contracts; host services; recovery fixtures; coverage mappings.
Dependencies: 018.
Roles: rust_engine, build_reliability, macos_ui, reviewer.

## Outcome

Diagnosable failures and storage operations that preserve drawings.

## Contracts and acceptance

Read [architecture](../docs/architecture.md), [quality](../docs/quality.md), and the
[refactoring sequence](../docs/refactoring-plan.md). One owner changes shared contracts.

- Specify events, correlation IDs, bounded sinks/rotation/redaction and dropped-event counts; no per-sample production logs.
- Move costly encoding/I/O off ink path with immutable snapshots, session/revision/request identity and same-destination write ordering.
- Define autosave/checkpoint frequency/retention/recovery UI and provider-specific durability before implementation.
- Inject disk/access/log failure, corrupt/oversized import, delayed saves, close/import callbacks and disposable process-kill scenarios.

## Required evidence

Name affected acceptance/invariant IDs, executable test mappings, exact source/diff and
binary identity, platform/toolchain/device, commands, results and artifacts. Run applicable
current integration checks until Node parity is established; then the verified replacements.
Failures, skips, flakes and unsupported hardware remain visible. Read-only review is not
runtime evidence. Writers use isolated worktrees and do not revert other workers' changes.
No recursive delegation; at most two writers/three subagents and one feature in flight.

## Completion boundary

Failure retention, stale-save/write ordering, recovery and logging-sink tests; no automatic hosted telemetry or network/auth expansion.

Fresh review follows substantive implementation and material fixes. Task status becomes
review when evidence is ready; only owner acceptance permits done. No main merge or release.

