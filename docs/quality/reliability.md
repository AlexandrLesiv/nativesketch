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

## Implemented background lanes (task 023 foundation)

`node scripts/check-reliability.ts --lane advisories|miri|asan --output NEW_DIRECTORY`
uses preinstalled project tools in `.build-cache/toolchains`; `--tools PATH` selects an
existing project cache for an isolated checkout. It never installs tools or refreshes
advisories silently. Create the output parent first; an existing output directory is
rejected, so retries preserve earlier evidence. Each command records argv, bounded logs,
failures, source/diff identity and tool hashes. Successful execution also requires complete
expected output and unchanged source. These lanes are separate from the normal fast gate.

Pins live in `quality/reliability-tools.json`: cargo-deny 0.20.2 and nightly-2026-10-15;
normal application builds retain stable Rust 1.99.0. Advisory checks use locked dependencies,
all features, no ignores, denied yanked crates and seven-day database staleness enforcement.
They run offline against the explicitly provisioned database and record its commit/date/age.
An outdated or missing database fails; updating it is a separate explicit provisioning step.
License policy/compliance is not inferred from advisories.

Miri and ASan currently require macOS arm64 and run exactly the six legacy `sketch-ffi`
library tests. ASan instruments Rust/core/std with leak detection; Miri keeps default
isolation. Separate `target/reliability-*` caches avoid contamination of ordinary builds.
No Skia, native UI, JNI, Wasm, other-host or new portable-facade safety is certified here.
Bounded property/fuzz targets, package acceptance and native profiling remain separate work.
