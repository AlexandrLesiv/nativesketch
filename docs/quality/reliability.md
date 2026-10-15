# Diagnostics, recovery and release evidence

Target obligations, implemented incrementally in tasks 017/021/023/024. No new logging agent
or telemetry service is required. Default to local, redacted, bounded diagnostics.

## Diagnostic contract

Rust structured tracing events/spans plus host adapters feed native logs and a bounded
diagnostic export. Include build revision, opaque document-session ID, operation/request ID,
revision, subsystem, stable event/error code and duration. Correlate input batch -> interaction
-> scene -> frame, and save request -> snapshot -> storage result -> acknowledgement.
Use monotonic duration clocks and explicitly map native/Rust/GPU clock domains.

Log operation outcomes/failures at the handling boundary once, with context. No silent
catch-and-ignore, raw document contents, coordinates, secrets or private paths by default.
Avoid per-sample production logs; use aggregated counters or opt-in bounded traces.
Measure overhead; cap/rotate buffers, expose dropped-event counts and rate-limit repetition.
Logging failure cannot block drawing or saving. Test redaction and a failing/full log sink.
Crash records/symbols supplement logs; queued events may not flush before a crash.

Native destinations: Apple unified logging/signposts, Android logs/Perfetto markers, Windows
structured logs/ETW as appropriate, Linux journal/file diagnostics, browser bounded export
and performance markers. Hosted crash collection is a later explicit decision, not a dependency.

Enforcement: each effect/failure case names its event, user-visible handling and test. Inject
failure and assert structured fields/outcome, not brittle full message text. build_reliability
owns sink/schema infrastructure; implementers instrument; reviewer checks diagnosability.

## Recovery matrix

| Failure | Required behavior / evidence |
| --- | --- |
| Interrupted save | Prior valid copy survives where atomic replacement is supported; test checkpoint interruption |
| Disk full / access revoked | Preserve in-memory edits/dirty state; retry or alternate destination |
| Corrupt/oversized/unsupported file | Validate bounded candidate before replacing current document |
| Process death | Versioned checkpoint/autosave recovery; do not rely on shutdown callback |
| Surface/context loss | Rebuild caches from authoritative document; reject stale callbacks |
| Memory pressure | Evict disposable resources and bound queues; preserve content |
| Input overload | Ordered samples, explicit tested error/cancel policy; no silent corruption |
| Stale save/source result | Session/revision/request check; no false clean or older overwrite |
| FFI panic/error | No cross-boundary unwind; defined invalid-handle recovery |
| Future network failure | Offline content retained, bounded retry only for safe/idempotent operations |

Document provider APIs cannot universally provide atomic replacement. Specify capability,
partial-copy cleanup and user recovery rather than claiming desktop path guarantees there.
Preserve browser download-copy dirty semantics. Define checkpoint frequency, retention,
recovery UI and durability guarantees in task 021 before implementation; don't promise
zero-loss across power failure. Process-kill fixtures use disposable documents only.

## Packaging and security

Test the built/packaged artifact in a clean environment: required assets/native libraries,
launch, supported old-document migration, upgrade with local content, offline operation,
permissions and distribution-specific signing. Never perform release publication implicitly.
Retain dSYM/PDB/native symbols and appropriate web source maps with exact binary identities;
protect private diagnostic content and limit artifact retention. Build a dependency/license
inventory, scan secrets, pin inputs and record reproducible build commands.
Validate untrusted data lengths/resource budgets before allocation and decompression.

## Rollout

Refactor stage: recovery fixtures, diagnostic contracts, dependency checks and relevant
runtime diagnostics. Platform stage: actual E2E, lifecycle/accessibility and package smoke.
Release stage: sustained/hardware regressions, symbols, upgrade/rollback and optional field
diagnostics. Record unimplemented checks in coverage; never imply this document installs them.
