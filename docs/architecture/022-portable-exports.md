# Portable export boundary v2 — signature freeze

> Migration update (2026-10-16): follow [typed bindings migration](022-typed-bindings-migration.md).
> Its owner-selected typed FFI/generated declarations supersede custom packet/layout
> prescriptions here. Preserve semantic/provider/lifetime requirements; existing code and
> historical evidence are not automatically migrated or invalidated. Do not expand packet consumers.


Task022 continuation after reviewed wire6df14a2. This boundary adds no platform consumer,
renderer or product capability. Native-v1, ABI2, Wasm-v1 and document bytes remain unchanged.
The shared registry owns objects; `sketch-app-wire` remains the only editing/work dispatcher.
Approved crate split: safe `sketch-app-bridge` (wire dependency), `sketch-app-ffi` and
`sketch-app-wasm` (bridge dependency). The latter two contain only platform export mechanics.

## Native signatures and status

`NsV2Handle` is u64, never a pointer or Rust discriminant. Zero is absent; issued IDs never
repeat across kinds/apps in this loaded runtime. `NsV2Error` contains exactly72 bytes, the
wire error frame. Functions return the explicit wire status u32. Status0 includes accepted
application failures encoded in Outcome/Failure; a nonzero status rejects the transport
operation and transfers nothing unless an explicitly named terminal-retirement call succeeds.
Error records require caller memory only, never registry allocation. Success zeros the error.

```c
uint32_t ns_v2_app_new(const uint8_t *packet, size_t len,
                     NsV2Handle *out_app, NsV2Error *error);
uint32_t ns_v2_app_call(NsV2Handle app, const uint8_t *packet, size_t len,
                      NsV2Handle *out_reply, NsV2Error *error);
uint32_t ns_v2_app_take_work(NsV2Handle app, const uint8_t *packet, size_t len,
                           NsV2Handle *out_work, NsV2Handle *out_reply, NsV2Error *error);
uint32_t ns_v2_work_copy_input(NsV2Handle work, const uint8_t *bytes, size_t len,
                             NsV2Handle *out_input, NsV2Error *error);
uint32_t ns_v2_work_call(NsV2Handle *inout_work, const uint8_t *packet, size_t len,
                       NsV2Handle *inout_input, NsV2Handle *out_result,
                       NsV2Handle *out_reply, NsV2Error *error);
uint32_t ns_v2_app_complete(NsV2Handle app, const uint8_t *packet, size_t len,
                          NsV2Handle *inout_result, NsV2Handle *out_reply, NsV2Error *error);
uint32_t ns_v2_input_free(NsV2Handle *inout_input, NsV2Error *error);
uint32_t ns_v2_buffer_len(NsV2Handle buffer, size_t *out_len, NsV2Error *error);
uint32_t ns_v2_buffer_copy(NsV2Handle buffer, size_t offset, uint8_t *destination,
                         size_t capacity, size_t *out_written, NsV2Error *error);
uint32_t ns_v2_buffer_free(NsV2Handle *inout_buffer, NsV2Error *error);
uint32_t ns_v2_app_free(NsV2Handle *inout_app, NsV2Error *error);
uint32_t ns_v2_result_retire_stopped(NsV2Handle *inout_result, NsV2Error *error);
```

C validity preconditions: every nonnull range is live, readable/writable as specified,
correctly aligned for scalar outputs, and nonoverlapping with other call inputs/outputs.
Null bytes are allowed only for length0. Scalar/error/handle output pointers must be nonnull.
An arbitrary invalid nonnull pointer is undefined C caller behavior; the API does not promise
to catch memory faults or validate foreign address ranges. There is no retained caller pointer.
Output handles/scalars must initially be0; otherwise reject before mutation. On rejection all
input/output handles and data counts remain unchanged; the error record is filled. Callers
must not concurrently access the same handle except to receive a documented Busy rejection.

## Transfer matrix

| Call | Accepted transfer | Rejection |
| --- | --- | --- |
| app_new | New app in out_app | No owner stored |
| app_call | New reply buffer | Existing app unchanged by transport rejection |
| app_take_work | Reply and optional issued Work | No work consumed |
| work_copy_input | Copied bounded InputBytes | Work and caller bytes unchanged |
| work_call100/101/107 | Reply; Encode failure returns Result and consumes Work; otherwise Work retained | Work/input retained |
| work_call102/103/104/105/106 | Reply and actual Result; consume Work; only102 consumes Input, others require no Input | Work/input retained |
| app_complete | Consume actual Result; clear its handle; new reply | Result retained |
| buffer/input free | Remove exactly one registered object; clear handle | No transfer |
| app_free | Remove owner only when typed shutdown_ready and no registry objects remain | Busy/Poisoned, owner retained |
| result_retire_stopped | Explicitly discard actual returned Result after owner-thread exit | No transfer while owner exists |

All needed registry slots/IDs/reply capacity are reserved before destructive typed calls.
Unused reservations roll back; IDs are burned, never reused. A wrong-kind object is rejected
without casting it. There is no ordinary result_free, active-work drop or forced successful
save. Provider work/result objects remain uniquely owned and carry actual typed proof.
Copying bytes is not proof of successful save/read. Zero-length/EOF copies succeed with0
written; offset beyond the buffer length rejects. A copy never truncates an entire packet
silently: callers use buffer_len and repeat explicit bounded copy operations as necessary.

## Thread ownership and terminal states

The native app lives only in its creating thread's local registry, preserving WireApp !Send.
Global app metadata contains owner ThreadId and terminal flags, never a WireApp reference,
closure, TLS pointer or owner storage. Metadata lookup checks the current thread before local
access. Reentrant owner access and concurrent checkout of one work/result reject Busy.
Global registry locks cover short ownership changes only, never encoding/decoding/provider I/O.
Work, Result, InputBytes and Reply are serially transferable Send objects.

Owner TLS destruction removes its app registration, marks shared metadata OwnerStopped, and
drops the local app. A worker cannot recover that editor through global metadata; new calls
using the removed app handle are Stale. Already issued work may finish/fail truthfully. Its
actual returned result can then be explicitly retired with result_retire_stopped; this gives
no save acknowledgment, clean-state change or navigation authorization. Hosts retain native
provider access until real work terminates and own their terminal grant cleanup. No hidden
callback claims host cleanup after its UI owner has disappeared.

Every export catches Rust unwind before C/JS return. A panic quarantines the affected chain;
normal app mutation and normal app_free then reject Poisoned. Independently owned buffers
can still be released. A lost unique proof does not become a synthetic Failed/Written result
or a shutdown pass. Owner teardown is terminal, not a claim that the editor can be resumed.
Wasm traps/abort are instance-terminal and cannot be caught as ordinary Rust unwind.

## Wasm registry and memory

Wasm mirrors the same object kinds/transfers with nonreused u32 handles, checked before issue.
Wire u64 values stay exact bytes/BigInt. Calls use bounded upload objects for request and provider
bytes: upload_new(length), upload_view(handle), upload_free(handle). upload_view reports the
current pointer and fixed length together. Upload
length never changes. A request upload is borrowed for a call; PrepareRead consumes only the
separate app-budgeted InputBytes returned by work_copy_input. Invalid transfers retain inputs.

JS obtains fresh memory.buffer views after every allocation/memory.grow and before reading or
writing; no view survives await. Uploads/replies are owned until explicit free. Output status,
handles and fixed error bytes must be available without a new error allocation. A fixed scratch
result record per synchronous call may be copied immediately; it must not be retained across
another call. A Worker/another Wasm instance cannot use this instance's numeric handles.
Initial support is single-owner synchronous Wasm, not shared-memory threading or OffscreenCanvas.

## Required evidence and limits

Registry limits are4096 total live/reserved objects and16 apps per runtime. Uploads additionally
have64 slots/128MiB aggregate and maximum64MiB+1 per object; control packets remain <=1MiB.
WireApp uses its existing default TransportLimits. All registry/output reservations precede
transfer; a freed object releases its slot, never its identity. Native IDs exhaust before u64
wrap; Wasm IDs exhaust before exceeding u32::MAX. Metadata is bounded by live objects, not an
unbounded tombstone table. Unknown already-freed handles are Stale. App-free and buffer/input/
upload free need no new registry object, so exhaustion does not prevent ordinary cleanup.

Wasm functions return status:u32 and publish one128-byte scratch record, copied immediately:
status at0, consumed_flags at4, primary_object at8, reply at12, work at16, result at20,
length_or_written at24, input at28 (all u32); fixed error frame at32..104; zero reserved104..128.
Flags: work1/input2/result4/app8/buffer16/upload32. On failure flags0 and no new owned outputs;
caller-owned scalar handles remain the caller's values. `ns_v2_wasm_result_ptr()->u32` returns
the scratch address without changing it. Other functions use the prefix ns_v2_wasm_:

```text
upload_new(length), upload_view(upload), upload_free(upload)
app_new(packet_upload), app_call(app,packet_upload), app_take_work(app,packet_upload)
work_copy_input(work,source_upload), work_call(work,packet_upload,input_or0)
app_complete(app,packet_upload,result), result_retire_stopped(result)
input_free(input), buffer_len(buffer), buffer_copy(buffer,offset,destination_upload,capacity)
buffer_free(buffer), app_free(app)
```

All arguments are u32. upload_view sets primary_object to its current linear-memory byte
offset and length_or_written to length; this is a borrowed view, not another owned handle.
app_new/upload_new/work_copy_input return their owned object in primary_object. app_call/
app_complete return reply; app_take_work returns reply+optional work; work_call returns reply,
optional result and explicit consumed flags. buffer_copy writes only within the registered
destination upload and reports count. No unregistered raw caller memory is read by these
Wasm operations. Calling any function may grow memory; reacquire JS views afterward. Scratch
memory never serves as a trusted input and cannot fabricate an owned object.

OwnerStopped is distinct from panic quarantine: panic alone never enables stopped-result
retirement. Only the owner TLS destructor proves terminal exit. Explicit stopped retirement
of a real result acknowledges disposal only, never provider durability or grant release.

Acceptance includes a real C compiler/link caller, actual native worker-thread transfer,
wrong thread/app/kind, reentrancy/concurrency, zero-output preflight, malformed data, capacity,
double release, actual WorkResult return and shutdown drain, owner TLS teardown and quarantine.
Actual Wasm instantiated in JS must exercise memory.grow, stale views, copied uploads, output
exhaustion and corresponding transfer failures. Keep these portable/bridge tests separate from
native UI, provider/device acceptance and rendering. No native scene borrow is exposed here.

Fixed error records echo opcode/request for an exact bounded NSAP v2 frame, including
rejected scope, owner, thread, output-capacity or payload calls. Malformed framing and
packetless functions use opcode/request0. Correlation parsing never allocates or dispatches.

## Verification and consumer gaps

Run `node crates/sketch-app-ffi/tests/run.ts` with pinned Node and the installed
wasm32-unknown-unknown Rust target. Normal non-core macOS/Linux integration checks invoke
this background runner; core checks include its strict TypeScript, Oxlint and Oxfmt gates
and the Rust ownership tests. Each consumer execution writes source identity, binary hash,
host, result and log under artifacts/evidence/portable-exports/<unique-run>. Import that
run's discovery.json and execution.json with scripts/test-evidence.ts --input. These are
C/Wasm bridge checks, not browser/native UI or real provider acceptance.

No app consumes these exports yet. Existing native-v1/macOS, ABI2 and Wasm-v1 remain live.
JNI, Windows consumers, Wasm Worker transfer, native scene borrowing and browser CanvasKit
placement need later contracts and host evidence. A quarantined owner cannot resume or free
normally; host owner-thread/process teardown is terminal. No lost proof is reconstructed.
After real owner exit, host code must retain provider grants until actual work ends.

Before host migration, preserve existing chooser/save semantics explicitly: Android and Linux
currently choose a fresh destination for guarded Save, while Apple confirmed export can clear
matching dirty state without binding and prepares the file before its native export picker.
Whole-document Fit also needs a typed query instead of scanning viewport-culled scene pages.
These are pending consumer amendments, not new wire tags or host workarounds in v2 exports.
