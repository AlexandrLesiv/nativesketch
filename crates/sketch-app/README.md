# Portable application owner

Task 022 typed Rust boundary. This crate depends only on `sketch-core`; it does not
perform I/O, spawn workers, own native controls, or choose a renderer. Host transport
and platform adoption remain pending. Existing native-v1 storage/recovery stays intact.

## Public API and ownership

`PortableApp::new(AppLimits)` creates one non-Send/non-Sync mutation owner. Keep it on
one host executor. `create_session(SessionConfig)` starts an empty editor/viewer;
load documents through issued, bounded work rather than injecting an initial model.
A `SessionHandle` identifies the host session slot; replacement changes the core
`DocumentSessionId`. App-local opaque handles contain an owner identity and monotonic
u64 value; `get()` preserves integer precision. Exhaustion fails before mutation.

- `dispatch(session, context, Intent)` accepts Edit, Action, Input, or File. Exact caller
  context is validated by the core. Samples are borrowed only for the synchronous call.
- `projection`, `availability`, `selected_bounds`, and `scene` delegate core queries.
  Availability is advisory; actual dispatch repeats context/geometry/capacity validation.
  Scene caches are supplied by the host and remain disposable, renderer-independent data.
- `reply` resolves an issued dialog continuation. Raw Source/Destination decisions are
  rejected here: use `supply_resource` to transfer a retained native resource lease.
- `supply_resource` and read-only `begin_view_source` take a `ResourceDescriptor` with
  opaque provider/resource/canonical identities, access capabilities, and generation.
  Only an Ok return transfers the lease, including an Ok outcome carrying a core failure.
  Err leaves the native resource owned by the host and emits no release.
- `take_work` transfers a unique non-Clone `WorkItem`. The host chooses an executor;
  encoding, full decode/validation, and large-state disposal belong on that worker.
- `complete_work(&mut Option<WorkResult>)` accepts only an actually issued result.
  Rejection retains the Option; success consumes it and stores a ready result without
  mutating the displayed editor. `apply_ready(session, operation)` applies it on owner.
- `cancel_operation` uses core cancellation policy. An active write remains globally
  ordered until its actual provider completion; a timeout or dropped work is not proof
  of termination. Late reads cannot install after a newer source/cancel/replacement.
- `detach_session` preserves in-flight work. Apply its real results and drain retirement
  work and releases. `stats().shutdown_ready()` is false until all owned work is settled.
- `take_release()` returns each accepted resource lease once. The host must validate the
  descriptor generation against its retained native resource before ending access.
- `take_diagnostic()` drains typed, redacted boundary events. The ring is bounded by the
  operation limit; `stats().diagnostic_drops` records eviction. Hosts own logging sinks.
  No paths, text, document bytes, coordinates, or samples are included; ink emits no logs.

Provider/resource IDs are host-issued correlation, not security grants. Known aliases
share core destination identity. One SaveCoordinator serializes all writes across all
sessions even for unknown aliases; unknown aliases cannot prove shared dirty-state identity.
Each session retains its exact chosen binding lease, separately from canonical destination
identity. Accepted Save As replaces that lease; completed export aliases release independently.

## Issued work

`WorkItem` is Write, Read, PrepareNew, or Retire. Work objects and results are Send,
contain no live owner borrow, and must not be used concurrently.

`ProviderWrite::encode(self)` returns `Result<EncodedWrite, WorkResult>` and performs
at most one encoding. `EncodedWrite::bytes()` borrows its immutable buffer. After the
provider no longer uses bytes, consume `finish(outcome)`. A rejected report returns
`(AppError, EncodedWrite)` intact so the actual completion can still be supplied.

Confirmed providers may report Written only for their established durability. Report
Uncertain when truncation/replacement may have happened; Failed means replacement did
not happen. Only unconfirmed ExportCopy can report DispatchedUnconfirmed. It never
binds a save destination, marks clean, or satisfies a Save decision. Settling an existing
AwaitWrites barrier rechecks navigation; dirty content still requires confirmation.

`ProviderRead::maximum_bytes()` is the read limit. Host reads at most limit+1, then
`prepare(Vec<u8>)` validates the entire candidate or returns an issued failure. The
provider must enforce the read bound before allocating untrusted input. Busy install
retains the prepared candidate; stale candidates become explicit retirement work.
`ProviderNew::prepare()` creates an issued empty replacement; `RetiredState::dispose()`
drops large retired editors/candidates on the worker and returns a completion.

## Bounds and limitations

Default caps: 16 sessions, 32 operations, 64 resource leases, 32 encoded/read reservations,
128 MiB aggregate reserved bytes, 64 MiB per document, 16 outstanding saves, and 128 MiB
snapshot budget. Workers retain reservation permits; handing out work does not subtract
its memory. Write admission reserves the document cap before encoding, then shrinks to
actual encoded length. Encoded/read buffers, snapshots, decoded candidates, and scene
caches are separate bounds, not a measured total process memory cap. Prepared candidates
are bounded by operation count and document limits; their decode reservation ends on worker.

No provider-stall timeout fabricates a save result. Abandoned issued work deliberately
leaves ordering/shutdown busy. Hosts must retain and finish/fail every handed-out item.
Operation IDs are consumed even for some rejected admissions; they never resurrect.
Recovery-source navigation remains Unsupported here; native-v1 owns current recovery.
No v2 wire/C/JNI/Wasm transport, portable renderer adapter, or native UI acceptance is
claimed by these tests. Skia objects never enter the owner or document state.

## Verification mapping

All cases are Rust owner/provider-contract tests, not platform UI E2E.

| Test module | Invariants and behavior |
| --- | --- |
| `tests/saves.rs` | SAVE-001/003: snapshot revisions, alias ordering, failure/uncertainty, active cancellation, download/navigation policy |
| `tests/navigation.rs` | SAVE-001, LOAD-001: bound/unbound Save decision failure/uncertainty, usable retry/discard |
| `tests/loads.rs` | LOAD-001, ASYNC-001, SOURCE-001: corruption, busy retry, stale edit/source, generation retention |
| `tests/budgets.rs` | FFI-001: worker-held permits, preflight limits, oversized encoding/read, lease transfer |
| `tests/lifecycle.rs` | FFI-001, ASYNC-001: rejected ownership, detached receipts, late cancellation, abandoned work |
| `tests/aliases.rs` | FFI-001, SAVE-003: repeated same-file leases, exact-once release, retained binding after failure/cancel |
| `tests/identity.rs` | FFI-001, ASYNC-001: >2^53/high-bit IDs, exhaustion atomicity, wrong-kind recovery, actual worker transfer |
| `tests/viewer.rs` | VIEW-001: all mutation intent families and unconfirmed Save As rejection |
| `tests/diagnostics.rs` | Bounded opaque diagnostics, failure correlation, no per-sample production records |

Run `cargo test -p sketch-app`, `cargo clippy -p sketch-app --all-targets -- -D warnings`,
and `cargo fmt -p sketch-app -- --check`. The root dependency gate permits only sketch-core.
