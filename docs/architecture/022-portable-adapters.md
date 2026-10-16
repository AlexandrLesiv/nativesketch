# Task 022 implementation contract — portable application adapters

Implementation design approved by the lead, 2026-10-16; starting source `8b3e3e3`.
The read-only contract audit used `e117245`/`06b0c4d`; subsequent changes are macOS
rendering verification repairs. The designer reviewed action and viewer interaction policy.
This document defines phased implementation, not implemented capability or host acceptance.
The lead approves validation-before-cancellation and an explicitly unconfirmed ExportCopy
outcome; neither may acknowledge persistence or introduce new user controls. Concrete wire
layouts freeze only after the typed Rust facade passes its ownership and failure tests.

Task 020's local renderer is implemented and inspected; final native/release verification
is blocked by a system dialog that computer control is not permitted to access. The owner
explicitly authorized independent work when permissions block a lane. Task 022 proceeds
as downstream preparation; this does not mark task 020 or any platform accepted.

## 1. Scope and decisions to freeze

Implement the already approved editor/viewer features on the existing hosts through the
same Rust application boundary. Retain native presentation, controls, accessibility,
localization, focus, dialogs and each host's current viewport policy. No new color controls,
agent transport, cloud service, immersive interface or editing on viewers.

1. Add a small portable `sketch-app` crate, depending on `sketch-core`, with no filesystem,
   mandatory executor, native widgets or renderer. Its purpose is ownership, bounded work
   transport and conversion of issued results into existing application commands.
2. Keep all editing/navigation/save policy in `sketch-core::application`. Add shared compound
   actions there; do not add a second navigation or save state machine in the facade.
3. Continue using `SaveCoordinator`, `WriteJob`, `WriteReceipt`, `SaveCompletion`,
   `ReplacementTicket` and `PreparedReplacement`. Never reconstruct their private tokens
   from foreign fields. Foreign completion consumes an actually issued Rust work object.
4. Add portable protocol v2 and C functions with a distinct `sketch_app_v2_*` prefix.
   Preserve legacy `sketch-ffi` ABI2, Wasm protocol1 and production native session v1 while
   callers migrate. Protocol versions are independent, not one global version counter.
5. Retain macOS NativeRuntime storage/recovery implementation initially. Route its new
   compound action through the same core command; do not replace its proven service
   implementation merely to adopt the portable transport.
6. Start with one app owner, a bounded session table, one SaveCoordinator and one active
   write for that app. This preserves global write order even when providers alias files.
7. A provider capability is an opaque host-owned resource, never a path embedded in the
   document. A browser download has no confirmed persistence acknowledgment.

## 2. Existing seams that must be reused

| Concern | Existing owner and contract |
| --- | --- |
| Mutation and native projections | `crates/sketch-core/src/application/routing/{types,dispatch}.rs`, `ApplicationSession::dispatch` |
| Compound primitive validation | `application/session/{dispatch,guard}.rs`; low-level active Undo remains rejected |
| Save capacity/order | `application/storage/{admission,lifecycle}.rs`; one active WriteJob; finished jobs count until consumed |
| Revision/binding acknowledgment | `application/session/storage.rs`; older writes cannot mark current edits clean |
| Load validation/install | `application/routing/replacement.rs`; issued ticket, fully validated candidate, session/revision/busy checks |
| Native local provider | `crates/sketch-services/src/native_runtime/*`; retain grant/recovery/retirement guarantees |
| Native v1 bridge | `crates/sketch-native-ffi/include/sketch_session_v1.h` and existing real C consumer tests |
| macOS compound duplication | `apps/macos/Sources/NativeSketch/EditorActions.swift` |
| Legacy Wasm | `crates/sketch-wasm/src/{command,dispatch,registry}.rs`; retain until v2 conformance passes |

`SaveCoordinator::start_next` already enforces write serialization. The facade must not
maintain a parallel queued/active/finished save state machine. Its work table only records
transport ownership: pending, handed out, result ready, consumed. Core owns save semantics.

## 3. Shared compound actions

Add `ApplicationCommand::Action(EditorAction)` and an application-level availability query.
The existing `Edit(EditorCommand)` remains the precise gesture/transaction boundary.
Suggested concrete Rust enum:

```rust
pub enum EditorAction {
    ChooseTool(ToolId),
    CycleSelection(CycleDirection), // Previous or Next; no arbitrary i32 steps
    DeleteSelection,
    Undo,
    Redo,
    ScaleSelection { factor: ScaleFactor, anchor: ScaleAnchor },
}
pub enum ScaleAnchor { SelectionCenter, SelectionTopLeft }
pub enum CycleDirection { Previous, Next }
```

`ScaleFactor` is a validated newtype created in Rust. Preserve currently accepted application
range (positive through 100, corresponding to macOS 10000%); validate composed transform
limits as well. Native text parses user-localized text and reports field errors; Rust repeats
semantic/range validation. No native host computes selection center or runs resize_begin /
resize_update / resize_commit for the one-shot scale control.

| Action | Accepted semantic operation |
| --- | --- |
| ChooseTool | Validate tool/capability, cancel current interaction, choose tool |
| CycleSelection | Validate direction and nonempty committed document, cancel interaction, choose Select, cycle committed selection |
| DeleteSelection | Validate capability and selected committed object, cancel interaction, delete once |
| Undo / Redo | Validate capability/history against committed state, cancel interaction, apply history operation |
| ScaleSelection | Validate factor, selection and resulting transform, cancel interaction, apply one committed transform anchored at the requested committed selection center or transformed top-left |

Invalid action parameters, stale context, denied capability and unavailable history must
leave draft, selection, settings and committed content unchanged. Validate against the state
after hypothetical cancellation without actually canceling it. Reserve generation/interaction/
revision capacity before mutation. Do not implement general arbitrary command batches or
clone the whole document as rollback machinery. Add a narrow prepared-action path within
EditorSession for the accepted sequence, sharing the existing transform/history routines.
The implementation must demonstrate that no recoverable failure remains after cancellation;
if a primitive still can fail, factor its validation/preparation before applying the action.

A successful compound action emits one final projection/outcome. The temporary intermediate
Select/tool/transform state is not presented. Scale creates exactly one history operation;
no-op/cancel remains zero. Aggregate invalidation/changes across the entire action. Preserve
primitive active Undo rejection (`UNDO-002`); explicitly document that the new high-level Undo
intent has cancel-then-undo semantics. Requests are correlation IDs, not automatic dedup keys.

### macOS adoption without storage migration

Add `sketch_session_v1_action(runtime, session, context, action, outcome)` as an additive
function using a new tagged action DTO; do not reinterpret existing v1 primitive tags.
Its conversion calls `ApplicationCommand::Action` through `NativeRuntime::dispatch`.
Add `sketch_session_v1_action_availability`; menus use high-level availability for high-level
commands instead of primitive active-interaction rejection.

In `EditorActions.swift`, keep `fieldEditing.finishEditing()`, native error announcements,
first-responder text Undo/Redo/Delete priority and focus restoration. Replace successful
canvas Cancel+primitive sequences with one `.action(...)` call. The input adapter then clears
its local capture/gesture state from the returned idle interaction/projection, without sending
a second Rust cancellation. A failed action must retain local gesture ownership as well as
Rust draft state. This tightens the current Swift behavior, which cancels before discovering
a subsequent primitive failure; freeze this invalid-action preservation explicitly with the
lead/designer, rather than presenting it as byte-for-byte legacy behavior. Pointer capture loss, deactivation and coordinate-remap cancellation remain
explicit primitive Cancel inputs. Interactive handle dragging still uses Begin/Update/Commit.

## 4. Portable facade: concrete Rust ownership surface

Proposed `crates/sketch-app/src/lib.rs` public types (names freeze after lead approval):

```rust
pub struct PortableApp { /* sessions, SaveCoordinator, bounded transport tables */ }
pub struct AppLimits {
    pub sessions: usize,
    pub operations: usize,
    pub destinations: usize,
    pub buffers: usize,
    pub buffer_bytes: usize,
    pub document_bytes: usize,
    pub storage: StorageLimits,
}
pub struct SessionHandle(u64);
pub struct OperationId(u64);
pub struct DestinationHandle(u64);
pub struct BufferHandle(u64);
pub struct ProviderId(u32);
pub struct ProviderResourceId(u64);
pub struct SourceGeneration(u64);
```

All runtime IDs are nonzero, monotonic and never reused within a runtime; exhaustion fails
before mutation. SessionHandle is a stable host-window slot; DocumentSessionId changes on
replacement. Zero is reserved for absent values. DestinationHandle resolves to core
DestinationId internally. Do not let foreign callers construct core private IDs/tokens.

Minimum facade methods:

- `new(limits)`, `create_session(config)`, `projection(session)`, `availability(session, action)`.
- `dispatch(session, context, intent)` where intent is Edit, Action, Input or File only.
- `reply(session, context, effect_token, decision)` for issued native dialog continuations.
- `supply_resource(session, context, effect_token, resource)` for a selected source/destination.
- `take_work() -> Option<WorkItem>`; transfer unique owned work out of owner, no execution.
- `complete_work(WorkResult)`; validate issued operation, store ready result; no UI mutation.
- `apply_ready(session, operation) -> Outcome`; apply using existing ApplicationCommand.
- `cancel_operation(operation)` and `take_release() -> Option<ResourceRelease>`.
- `detach_session(session)` and `shutdown_status()`; Busy until owned work/resources retired.
- `scene(session, viewport, cache)` and selected-bounds query delegate existing read-only core.

Context includes exact current session and interaction; request is caller correlation. Validate
context at the owner boundary, never silently substitute the current session for a stale
caller. Ready completion is applied with the original issued operation metadata and a current
owner context; core ticket/save validation remains authoritative.

Default policy proposal: 16 sessions, 32 transport operations, 64 destination registrations,
32 owned buffers, 128 MiB aggregate buffers, 64 MiB per document, 16 queued/outstanding saves,
128 MiB snapshot budget. These are explicit configurable safety caps, not performance promises.
Completed-but-unconsumed operations/buffers count. In-flight buffers transferred to workers
still count against the issuing app budget: attach an owned reservation permit until final
release, rather than subtracting bytes when they leave the owner table. Snapshot, scene and
encoded-byte budgets are separate; their sum is not a measured process memory limit. Reserve
up to document_bytes for an encoding job before starting it, then account actual encoded size;
if that reservation cannot be acquired, leave the job pending without encoding.
Reserve output/event/release capacity before
accepting an action that must emit them. One release slot is reserved with each resource lease;
release notification cannot be silently dropped because the event queue is full.

An app owns a single mutation executor, but this crate starts no thread and performs no I/O.
Bounded snapshot capture occurs only on explicit file/idle actions, never pen callbacks.
Encoding, validation and disposal of large retired state are explicit work items. Do not drop
a large retired editor as a side effect of input dispatch or completion notification.

## 5. Provider resource and completion contract

```rust
pub struct ResourceDescriptor {
    pub provider: ProviderId,
    pub resource: ProviderResourceId,
    pub canonical: ProviderResourceId,
    pub access: ResourceAccess,
    pub generation: u64,
}
pub struct ResourceAccess {
    pub read: bool,
    pub write: bool,
    pub confirmed_write: bool,
}
```

The host resource ID indexes a retained URL/security scope, Android document URI/permission,
Windows StorageFile, GTK portal object or browser FileSystemFileHandle. `canonical` identifies
aliases known by that provider; if equivalence cannot be proven, issue a distinct identity.
The initial global single-write policy protects physical write order regardless of alias
knowledge. It does not prove dirty-state identity across unknown aliases: document that limit.
No raw native object, URI, path or file contents enters logs or persisted document state.
Resource generation is validated before every completion and release. Registration transfers
an access lease only on success; rejected calls leave ownership with the host.

`confirmed_write=false` is suitable for read-only/import/download-only resources. Save/SaveAs
must not bind such a resource. ExportCopy may produce bytes for an unconfirmed download, but
the result is not Written and must not mark clean. Preserve browser's current honest behavior;
a dedicated `ExportDispatchedUnconfirmed` host notification may report dispatch, not persistence.
Do not map download dispatch to `ProviderConfirmed` or pretend it is a failed physical write.
Concrete proposed additive core outcome: `WriteOutcome::DispatchedUnconfirmed`, with matching
`SaveResult::DispatchedUnconfirmed` / `DispatchedAfterCancellation` and acknowledgment
`Acknowledgement::CopyDispatched`. Accept only an issued ExportCopy job whose provider is
unconfirmed; reject it for Save/SaveAs/Checkpoint before consuming the issued work. It releases
the operation without binding a destination or acknowledging a revision. Settling an existing
AwaitWrites barrier may re-evaluate navigation, but dirty content still requires the normal
Save/Discard/Cancel decision; unconfirmed export cannot satisfy Save.
The core coordinator repeats the purpose check. This addition needs its own fault/compatibility
tests and exhaustive-match updates; preserve old numeric ABI values and allocate new v2 codes.

### Work objects, not caller-authored success tokens

```rust
pub enum WorkItem {
    Write(ProviderWrite),
    Read(ProviderRead),
    PrepareNew(ProviderNew),
    Retire(RetiredState),
}
// Private fields own the original WriteJob/ReplacementTicket and issued operation metadata.
// Methods consume the object when producing a result; none implement Clone.
```

`ProviderWrite::encode(self)` runs off the input path and returns an owned EncodedWrite that
contains the original WriteJob plus a bounded encoded byte buffer, or an issued failure result.
The host writes those bytes through its retained resource, then consumes EncodedWrite via
`finish(WriteOutcome)`. Internally this calls the actual `WriteJob::complete`. The owner calls
SaveCoordinator::finish, take_completion and ApplicationCommand::CompleteSave; hosts never
call mark_saved or choose SavedCurrent/SavedOlder themselves.

`ProviderRead` owns the original ReplacementTicket and source resource. Host reads no more
than document_bytes+1, fails oversized input, then `prepare(bytes)` validates the entire
portable document and consumes ticket.prepare(Some(validated)). Decode failure consumes
`ticket.fail`; current document remains intact. `PrepareNew` calls ticket.prepare(None).
Prepared candidates return to owner; busy interaction defers apply without consuming them;
stale session/revision invalidates and retires them. No partial import state is exposed.

Native provider operations report:

- `Written(FileAndDirectorySynced)` only when the adapter actually established both syncs.
- `Written(DirectorySyncUnconfirmed)` for the existing local-provider qualification.
- `Written(ProviderConfirmed)` only after the platform provider's documented completion.
- `Failed(error)` only when replacement definitely did not occur.
- `Uncertain(error)` when truncation, replacement or completion may have occurred.

An Android provider that opens a document in truncating mode cannot report Failed after a
partial write as though the original survived. Do not claim atomic replacement where its API
does not provide it. Task022 tests must include this provider failure class.

### Cancellation, capacity, teardown and stale delivery

- Before work starts, coordinator cancellation removes queued save snapshot and produces its
  existing CancelledBeforeStart completion. Cancelled replacement invalidates its continuation.
- Once a write is active, cancellation is a request, not evidence that I/O did not happen.
  Keep the issued job until its real receipt; core produces WrittenAfterCancellation/Uncertain.
- Do not start a later write while an earlier provider write may still be running. A timeout
  alone cannot safely release ordering. Surface provider-stalled state and retain bounded slot;
  only confirmed termination/completion can unblock writes. No infinite unbounded retry queue.
- Late read/preparation results cannot replace a newer session/source request. Dispose off the
  input path. Late write results still settle coordinator ownership and may invalidate old
  clean knowledge as existing core dictates; do not discard a receipt merely because UI closed.
- Duplicate/unknown/wrong-session/wrong-kind completion is rejected before consuming valid work.
  Retry of a rejected result retains its ownership. Successful consumption makes subsequent
  handles stale; IDs never resurrect after slot reuse.
- Detach keeps runtime-owned in-flight receipts alive. Busy shutdown is explicit; do not free a
  runtime under active callbacks. OS process termination is not a successful save acknowledgment.
- Failures create bounded correlated diagnostics with operation/session/request/revision/provider
  code and outcome class, never paths/content. No per-sample production logs.

## 6. C/JNI/Wasm transport v2

Use the same portable crate and one portable wire codec. Native typed wrappers remain idiomatic;
transport is not application policy. Add `crates/sketch-app/src/wire/` for deterministic
little-endian encoding/decoding shared by JNI and Wasm and usable by C tests.

Packet header: magic `NSAP` (4 bytes), major u16=2, minor u16=0, opcode u32, payload_bytes u32,
request_id u64. Fields are explicitly encoded; never memcpy Rust enum layout. Reject unknown
major, opcode, enum tags, nonzero reserved bits, invalid lengths, NaN/infinite coordinates,
invalid capabilities and oversize payload before mutation. Minor upgrades are additive only;
unknown operations remain Unsupported rather than guessed. Require exact known payload lengths
until optional-extension framing is deliberately implemented.

Stable initial operation groups: create/detach/projection/availability (1–4), primitive edit /
compound action / sample batch (10–12), file intent / effect reply / resource supply (20–22),
work take / complete / apply / cancel / release (30–34), scene descriptors / geometry copy (40–41),
viewer source begin (50). Numeric schemas and golden fixture bytes freeze in the first commit;
this grouping is not permission to invent features. Rust public APIs use typed enums, not numbers.

All identifiers/revisions/timestamps-as-integer-ticks are u64 fields, never f64. JavaScript uses
BigInt/DataView getBigUint64; Kotlin preserves bit patterns in Long, C# uses ulong, Swift UInt64.
C and JNI array counts use checked usize conversions; no uint64→double paths. Tests include
2^53+1, high-bit values, maximum legal IDs and exhaustion. Floating timestamp inputs remain
existing f64 seconds under the current input contract; do not silently change clock units.

Suggested C ownership API (exact structs in dedicated <=200-line headers):

```c
typedef struct SketchAppV2 SketchAppV2;
typedef struct SketchWorkV2 SketchWorkV2;
typedef struct SketchResultV2 SketchResultV2;
typedef struct SketchBufferV2 SketchBufferV2;
uint32_t sketch_app_v2_create(const SketchAppConfigV2 *, SketchAppV2 **);
uint32_t sketch_app_v2_call(SketchAppV2 *, const uint8_t *, uint32_t, SketchBufferV2 **);
uint32_t sketch_app_v2_input(SketchAppV2 *, uint64_t, SketchContextV2,
                           uint64_t, const SketchSampleV2 *, uint32_t, SketchOutcomeV2 *);
uint32_t sketch_app_v2_take_work(SketchAppV2 *, SketchWorkV2 **);
uint32_t sketch_work_v2_info(const SketchWorkV2 *, SketchWorkInfoV2 *);
uint32_t sketch_work_v2_encode(SketchWorkV2 *, SketchBufferV2 **);
uint32_t sketch_work_v2_prepare(SketchWorkV2 **, const uint8_t *, uint32_t, SketchResultV2 **);
uint32_t sketch_work_v2_finish(SketchWorkV2 **, SketchWriteOutcomeV2, SketchResultV2 **);
uint32_t sketch_app_v2_complete(SketchAppV2 *, SketchResultV2 **);
uint32_t sketch_buffer_v2_copy(const SketchBufferV2 *, uint8_t *, uint32_t, uint32_t *);
uint32_t sketch_buffer_v2_free(SketchBufferV2 **);
```

Complete ownership/lifetime/free/cancel functions must accompany these entry points, not be
left implicit. Native editor/app calls verify the creating owner thread. Unique work handles
may move to one worker thread after take; never use concurrently. Returned results move back
to app owner. Work owns no live editor references. Copy API permits managed runtimes to avoid
borrowing native memory across async suspension. Byte buffers are immutable once published.
`encode` is once-only/cached within that work object and returns at most one bounded buffer;
calling repeatedly must not allocate unlimited copies. Write finish releases its encode lease
only when provider no longer needs the bytes. Invalid calls preserve output arguments and
ownership. Panic poisoning and required unsafe caller obligations match existing native FFI.

The C ABI can expose compact typed action/projection wrappers over the wire implementation;
input takes borrowed samples for one synchronous call to avoid per-sample serialization copies.
Generated C headers are ABI documentation, not an authored C implementation. Android uses a
Rust JNI adapter crate; remove the owned C++ glue only after its replacement consumer passes.

Wasm exports use a distinct v2 prefix, u32 registry handles for objects/buffers, and identical
packet bytes. Wasm handles are transport-local, not document IDs. Never retain a JS typed-array
view across possible memory.grow; reacquire pointer/view and copy owned output before await.
Old protocol1 registry/exports remain intact until no shipped caller uses them. Each registry
has aggregate byte/entry limits and checked nonreused handles.

No mandatory Wasm threads or SharedArrayBuffer. Native workers consume moved work objects.
Browser I/O is asynchronous, but Wasm encode/decode CPU work does not automatically move off
main. Initial browser integration must explicitly choose one of: a Worker owning the whole v2
engine and queued commands, or bounded scheduled non-input CPU execution with its responsiveness
gap recorded. Do not claim background encoding from an async function. The recommended final
web slice is one dedicated engine Worker, ordinary message passing, ordered actual input,
coalesced repaint notifications and explicit queue overflow failure/cancellation. Add no
SharedArrayBuffer deployment requirement to GitHub Pages.

The implemented first browser slice deliberately uses the bounded main-thread option;
see the [adoption handoff](022-web-portable-adoption.md). File reads are asynchronous,
but encode/decode remains synchronous outside input callbacks. The later whole-engine
Worker is still pending; do not describe the current implementation as background decoding.

## 7. Read-only viewers and source ordering

Initial viewer session creation uses Capabilities { edit_document:false, export_document:false }
and Select defaults. Mutation denial must be enforced by Rust even if native UI exposes no edit
controls. Existing general Navigate(Open) currently requires edit capability; do not weaken it
for all sessions to make viewer loading work.

Add a narrow `ViewSource` application intent available only to read-only sessions. A facade
`begin_view_source(resource, source_generation)` obtains an issued core ReplacementTicket for
that session; source generation must increase and completion must match the latest issued
request. Reuse ticket validation/install and retirement, with no save binding and no dirty or
unsaved workflow for a viewer. This is source replacement, not permission to edit or export.
Make core ticket construction private; facade cannot manufacture an unchecked candidate.

Keep existing Watch phone-first/network-fallback/cache behavior and source revision policy.
A transport result is staged until both existing source policy and latest issued request accept
it; only then install. Phone queued is not received, server bytes are not trusted, stale network
response cannot replace a newer phone result. Preserve the last displayed valid document on
corrupt/offline/oversize failure. No new server/account implementation in this refactor.

## 8. Host adoption and renderer boundary

| Host | Adoption files/modules and service adapter |
| --- | --- |
| macOS | EditorActions / SessionCommand / action availability; native v1 remains local runtime and Metal renderer owner |
| iOS/iPadOS/visionOS | Extract `apps/ios/Sources/NativeSketch.swift` into composition, presenter, UIKit input, viewport, session bridge, coordinated file effects, lifecycle; portable v2 C adapter; Metal reuse requires separate target build/runtime evidence |
| Android/ChromeOS | Extract MainActivity.kt; Rust JNI over sketch-app, Kotlin Views presenters/input, ContentResolver resource provider, bitmap/surface owner; preserve existing desktop mode |
| Windows | Extract App.cs; v2 P/Invoke wrapper with owner-thread disposal, DispatcherQueue presenter, StorageFile provider; never finalize owner-affine app handles on GC finalizer thread |
| Linux | Extract main.rs; direct sketch-app API, GLib owner, GTK native controls, portal/provider work, viewport and texture presenter |
| Browser | Split main.ts/engine.ts, protocol-v2 Worker client, semantic controls, focus router, file effects, CanvasKit presenter; migrate nested tooling to root Oxc/Vite |
| tvOS/watchOS | Shared read-only source bridge + source coordinator/cache adapters; explicit viewer render/build exception until Skia target support is proven |

Shared immutable RenderScene/SceneCache stays renderer-independent. Renderer binding is a
separate adapter using session/generation/viewport keys; never call legacy Editor to obtain a
second mutable scene. CanvasKit needs a bounded scene handle with compact ordered descriptors
and geometry buffers keyed by geometry identity; reuse immutable geometry between frames and
release handles explicitly. Do not ship a giant flattened scene copy per pointer sample as the
new steady-state contract. Define renderer wire geometry IDs as facade-owned opaque values,
not process pointer values. Scene acquisition and serialization are budgeted outside input.

macOS renderer currently validates zoom .1–16 while Linux supports .05–32. Preserve each host's
accepted viewport range, separating product viewport policy from backend representability;
do not silently narrow Linux zoom or globally expand all platforms as part of extraction.
Portable CPU Skia raster + native blit is an acceptable documented initial Android/Linux/Windows
surface, with actual host acceptance and later measured GPU work; it is not GPU parity.
Skia's pinned build support has no explicit tvOS/watchOS platform case. Preserve existing viewer
rendering with an explicit exception/build probe; do not report unsupported targets migrated.

## 9. Ownership and implementation slices

Single contract owner changes this schema; lead freezes exact commit before concurrent writers.
At most two implementation writers, isolated worktrees; contracts before consuming adapters.
New modules/files/tests <=200 physical lines after formatter. Keep responsibility-based names.

1. **Core actions and viewer source contract — rust_engine.** Own core application action
   enum/preparation/dispatch/availability, viewer routing and tests. No platform writes.
2. **Portable facade and work ownership — rust_engine, serialized after core contract.** Own
   new sketch-app crate, worker/result/buffer/resource registries, wire codec and tests. Expose
   typed Rust APIs before C/Wasm/JNI wrappers. Cargo manifests/lockfile owned by integrator.
3. **Native bridge — assigned Rust writer.** Own additive v1 action ABI, v2 C headers/converters,
   real C consumer fixtures, existing native-runtime Action routing. Keep services policy intact.
4. **macOS consumer — macos_ui.** Own only Swift compound action adoption and native regression
   tests. Designer reviews focus/action parity; reserved foreground tests required separately.
5. **Web consumer — assigned implementation writer.** Own Wasm v2 adapter and web extraction,
   CanvasKit adapter/worker/provider tests. One contract owner approves wire changes first.
6. **Apple editor/viewer consumer — assigned native writer.** UIKit extraction, coordinated file
   adapter, read-only source migration; preserve Watch source policy. Never co-edit macOS files.
7. **Android consumer — assigned implementation writer.** JNI Rust + Kotlin extraction/provider;
   remove bridge.cpp/CMake only after ABI replacement build and real JNI tests.
8. **Windows and Linux consumers — respective assigned workers, serialized if capacity needed.**
   No fake host acceptance from cross-compilation. Native controls preserved.
9. **Quality integration — build_reliability in its own writer slot.** Coverage registry,
   portable fixtures, host E2E runners/preparation modes, source/dependency/lint gates. It must
   not alter assertions or baselines to relabel missing host evidence as green.
10. **Fresh reviewer.** Independently test ownership, provider failure classes, stale/double
    completion, compound action invalidation and evidence; designer reviews native behavior.

Freeze checkpoints: (A) core action semantics and fixture bytes; (B) facade ownership/public
Rust API; (C) v2 packet schema and C ABI layout tests; (D) one complete host migration with
provider fault tests; (E) remaining consumers. Consumers do not start against changing tokens.
Mac actions may migrate after A+C without waiting for wholesale portable provider migration.

## 10. Required tests and closure evidence

Existing invariant IDs remain authoritative; suggested new test case names below need entries
in the executable coverage registry, not just this prose. Distinguish harness existence from
execution status and native E2E from injected replay.

| Group | Minimum executable cases | Invariants |
| --- | --- | --- |
| Core actions | active draft Undo cancels then undoes committed stroke; primitive Undo still rejects; invalid factor/direction/stale context preserves draft; scale once undo/redo; tool/cycle semantics; no-op history | INK-001, UNDO-001/002, ASYNC-001 |
| Input conformance | actual/prediction/correction fixtures; invalid trailing sample atomic; optional pressure; pointer ownership; batches equivalent across transports | INPUT-001/002, INK-002/003 |
| Provider saves | edit B while save A pending; delayed old binding; SaveAs failure; partial-write uncertain; active cancellation receipt; alias ordering; provider-stalled bounded queue; duplicate/wrong-kind results | SAVE-001/002/003, ASYNC-001, FFI-001 |
| Load/provider | oversize/corrupt/version-invalid candidate; newer edit while decode pending; busy apply retains result; source cancellation; access revocation; release exactly once | LOAD-001, ASYNC-001, FFI-001 |
| Resource budgets | full operation/output/release table before mutation; one-beyond byte cap; completed-but-unconsumed work; repeated encode; detached session receipts; handle exhaustion | FFI-001, ASYNC-001 |
| Viewers | every mutation rejected via C/JNI/Wasm/direct API; out-of-order phone/network results; corrupt latest candidate retains previous view; no saved-state mutation | VIEW-001, SOURCE-001, LOAD-001 |
| ABI/wire | actual C compile/link/run; Kotlin JNI load/call; Wasm v1 compatibility and v2 Node runtime; integer >2^53 and high-bit IDs; endian/golden packets; malformed lengths/tags; panic boundary | FFI-001 |
| Scene | same document/action stream generates equivalent scene and retained geometry IDs; no stale scene after replace; bounded handles; no all-geometry serialization on unchanged frames | RENDER-001, CACHE-001, ASYNC-001 |
| Native UI | text-field Undo/Delete/tool-key priority; menu/control same action result; drag→tool/history; SaveAs cancel/failure; import during save; lifecycle capture cancellation | FOCUS-001, CANCEL-001, SAVE-001 |

Use disposable documents/providers. Record exact source/build identity, platform/device,
commands/test IDs, pass/fail/skips, artifacts and unmodified failed seeds. Native test runner
currently primarily covers macOS: add real host lanes, not platform labels on replay tests.
Windows needs Windows, GTK needs a real Linux display stack, device-only pencil/Watch/ChromeOS
behavior remains a hardware gap. Browser Playwright is a real browser UI lane; it does not prove
native mobile/desktop acceptance. No desktop focus or GPU workload without a host reservation.

Task022 completion remains conformance + extraction ready for host acceptance. Task024 cannot
claim all-platform certification from compilation/replay. Report unexecuted platform lanes and
unsupported renderer targets explicitly. Owner approval still controls merge/release.
