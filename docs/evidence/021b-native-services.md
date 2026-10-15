# Task 021B: native service evidence

Tested source: `89e894e504ecb1ed2270cd823a5d94793094011e`, branch
`codex/storage-recovery-services`; implementation `66fb12e`, diagnostic-schema fixture
follow-up `89e894e`. Both commits were frozen before final checks. This phase provides
native service APIs; it does not claim the existing app adopted asynchronous persistence.

Host: macOS 27.0.1 (26A434), Apple Silicon; Rust 1.99.0, Node 24.22.0. Disposable local
fixtures ran in the background. No app was opened or focused. Node dependencies were
installed with `npm ci --ignore-scripts --offline`.

## Executed checks

`node scripts/check.ts --core` passed: 44 Node tests, 104 Rust tests, strict TypeScript,
Oxlint/Oxfmt, Rust format/Clippy, dependency and source-size gates. All 28 new service tests
passed. One child-writer fixture is intentionally ignored by normal enumeration and is
explicitly spawned by the two process-kill parent tests. It is not an unexecuted feature
acceptance case. Six existing oversized host files remain migration debt; every added
source/test complies with the 200-physical-line limit.

`node scripts/check.ts --platform wasm` passed actual Node/Wasm/native document-byte
conformance and 20 frozen protocol-v1 pairs. Existing ABI2/document tests passed in the
workspace suite. Native macOS build/ABI smoke is explicitly skipped by `--core`; no native
UI, Windows, Linux, mobile, provider/sandbox or physical-input acceptance ran in this phase.

`cargo test -p sketch-services -- --list` produced executable discovery, retained alongside
logs in ignored `artifacts/verification/021b/` in the isolated implementation worktree.
The lead owns acceptance-registry integration; this record does not mark task021 complete.

| Artifact | SHA-256 |
| --- | --- |
| `core-check.log` | `3d6b0d0b8140e33ab0145c9ce07f5ce93e8f48668f4c5fd922fd11cac428e36f` |
| `wasm-check.log` | `d86b690e6bcf8b4c11fbe7ec93a3b5b03764866c802424cb490c25be9d911f10` |
| `discovered-tests.log` | `01b6ec41ade16935f550a0e0afef8f6fd6eeb35fbe94b00aefd7a7ce5d98fef6` |
| `target/wasm32-unknown-unknown/release/sketch_wasm.wasm` | `297a97715234451bb3515822c0c31c3e531650d0a506c98f181e3731616c4a39` |

## Test coverage

Tests reside in `crates/sketch-services/tests`; discovered names are exact Cargo names.
Relevant invariants: SAVE-001/002/003, LOAD-001, ASYNC-001, with existing INK-002 and VIEW-001
coverage retained in task021A. These are service boundaries, not native UI E2E.

| Executable | Cases | Verified behavior |
| --- | ---: | --- |
| `checkpoint_schedule` | 2 | Debounce/max scheduling target, active-input deferral, explicit-save priority, coalescing, suspended retry and stale completion |
| `diagnostic_schema` | 1 | Frozen v1 event/subsystem/stage/failure/durability numbers and exact absent-field JSON fixture |
| `diagnostics` | 4 | Ring/rate limits, sink failure/panic, bounded rotation, oversized existing logs, correlated redacted failure/durability fields |
| `process_kill` | 2 + child fixture | Actual child termination at ordinary write/flush/replace boundaries and checkpoint generation publication |
| `recovery` | 5 | Corrupt latest fallback, malformed/oversized envelopes, quotas/ID collision, dirty unbound restoration/capability, symlink rejection |
| `recovery_budget` | 1 | Aggregate marker staging reservation, capacity refusal before mutation, unresolved data preserved |
| `recovery_retention` | 4 | Exact resolution visibility/expiry, corrupt marker cannot delete new content, isolated overfull-session management, valid unresolved fallback |
| `staged_store` | 4 | Pre-replace failures preserve target, cleanup failure observable, post-replace sync warning, missing parent/access and symlink rejection |
| `store_permissions` | 1 | Unix stage starts private; existing 0600/0640 permission bits survive replacement |
| `worker` | 3 | Owned snapshot/correlated receipt, post-replace panic uncertain outcome, checkpoint purpose/target isolation |
| `worker_lifecycle` | 1 | Bounded backpressure returns unique job; shutdown/Drop does not join an active blocked writer |

Process-kill tests terminate ordinary writers after write, file sync, before replacement
and after replacement. Checkpoint writers are terminated after write and after replacement
with an older valid generation already present. Parent processes validate old-or-new complete
content, discover/load a valid checkpoint and run bounded staging cleanup. These demonstrate
process interruption handling, not physical power-loss durability.

Independent review found and prompted fixes for resolved-candidate visibility/corrupt
resolution deletion, Unix permission widening, overfull-session isolation, missing failure
stage diagnostics and aggregate resolution-marker staging accounting. Final independent
assessment is reported separately by the reviewer after inspecting the frozen commits.

## Remaining integration and limits

See `docs/architecture/021-native-services.md` for public APIs and ownership. The typed
native bridge, native grant lifetime/destination mapping, asynchronous read job, shared
command router, recovery UI and diagnostic sink wiring remain task019 work. The host must
drain issued receipts during deferred shutdown. Early worker Drop is not a save promise.

Std-only filesystem checks reject observed symlinks but do not prevent hostile concurrent
path swaps. Local trusted directories and one exclusive owner are required. Standard Unix
permission bits are preserved; ACLs/xattrs/provider coordination and Windows replacement
semantics need native adapters and their own validation. Recovery checksums detect accidental
corruption, not malicious alteration. No universal power-loss or memory/latency bound is
claimed. Snapshot capture remains synchronous O(document) outside pen callbacks; worker
encoding/validation adds bounded temporary document buffers. No renderer changes or main
merge occurred.

## Integrated service and evidence verification

Clean integrated revision `929535759f7ea30ef93b9bad9686f33a548b511d` passed the complete
background integration command: 54 Node tests, 105 Rust tests, formatting/lint/type checks,
source/dependency gates, macOS build and headless Swift/Rust integration. One ignored child
writer remains explicitly exercised by its process-kill parent tests. A sandboxed attempt
at earlier `3d44f76` completed portable checks but Swift could not initialize its sandbox;
the authorized normal-host retry passed. Both logs remain local.

Independent evidence review caught and fixed three reporting gaps: native Cargo results
must identify the actual host, native keyboard mappings must tolerate absent host discovery,
and a worker shutdown claim needed an actual drain/acknowledgement regression. The new test
blocks an active write, queues another, requests nonblocking shutdown, rejects new admission,
then acknowledges both receipts in FIFO order and checks both saved files and final stop.
Early Drop remains explicitly different from draining accepted work.

Actual executable discovery/execution at `9295357` found 106 tests: 105 passed and the child
fixture was skipped as a standalone test. The coverage report has 184 acceptance/platform
rows, 25 passed and zero mapping issues. Eight narrow macOS service cases pass. Windows/Linux
service rows remain missing; native keyboard rows remain missing-discovery at this integrated
revision rather than importing historical native results as current passes. Overall coverage
is incomplete. Host-scoped Cargo IDs include the OS; portable IDs retain compatibility.

Fresh independent review found no remaining issues in these changes. It ran a real minimal
Cargo fixture for native/portable attribution, invalid declaration and skip behavior, ten
targeted Node tests, both worker lifecycle tests and simulated foreign-host mapping checks.
This is not Windows/Linux execution or native UI review.

Local raw records: `artifacts/verification/021b-integrated/` and
`artifacts/verification/021b-runner-independent/review-evidence.json`. The integrated run
does not replace the earlier native service API review or certify task019 adoption.
