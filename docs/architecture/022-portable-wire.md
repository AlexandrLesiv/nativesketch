# Portable wire v2.0

Lead-approved task 022 transport design; implementation acceptance is separate. This
contract wraps frozen sketch-app 42bd477. It changes no document format, legacy ABI2,
native-v1 or Wasm-v1. No C/JNI/Wasm exports or host consumer are added in this slice.
`sketch-app-wire` depends only on sketch-app and sketch-core. Native controls, providers,
executors, rendering and focus stay in hosts. Handwritten source/tests remain <=200 lines.

## Framing and validation

All fields are little-endian, tightly packed, with IEEE-754 f64. Never copy Rust/C layouts.
Header is 24 bytes: magic[4] NSAP, major:u16=2, minor:u16=0, opcode:u32,
payload_bytes:u32, request:u64. Response opcode sets bit31 and echoes request.
Payload length must match exactly. Requests with response bit, unknown version/opcode/tag,
nonzero reserved fields, unknown flags, malformed lengths, nonfinite floats or overflow
reject before mutation. Maximum control packet 1 MiB; sample count <=16384. Core repeats
semantic/range/batch validation. Parsing must not partially dispatch a valid prefix.

Response prefix: status:u32, reserved:u32=0, Failure (40 bytes). Failure fields are
 domain:u32, code:u32, operation:u32, reserved:u32, document_session:u64,
 request:u64, interaction:u64. Successful prefix is zero. Accepted application failure
remains OK with a Failure effect. A Rust WireError can encode a fixed 72-byte error packet
without allocating a registered output. Ordinary rejected calls retain output ownership.

Status numbers: OK0 InvalidArgument1 InvalidState2 Capacity3 IO4 Document5 Panic6 Stale7
Denied8 Busy9 Poisoned10 WrongOwner11 Stopped12 Unsupported13 NotFound14. These are explicit
v2 assignments matching familiar native-v1 numbers; they do not reuse its ABI declarations.

Failure codes below start at 1 in the named order; encode with exhaustive explicit matches:

| Domain | Codes |
| --- | --- |
| 1 Editor | InvalidValue, InvalidTransition, ReadOnly, StaleSession, StaleInteraction, ResourceLimit, CorruptDocument, ServiceFailure |
| 2 Storage | Busy, CapabilityDenied, InvalidBinding, BindingPending, ResourceLimit, UnknownToken, InvalidTransition |
| 3 Write | Encode, Access, Capacity, Write, Replace, WorkerStopped |
| 4 Router | InvalidEffect, StaleDocument, Busy, CapabilityDenied, NavigationRequired, ResourceLimit |
| 5 Scene | InvalidViewport, ResourceLimit |
| 6 Transport | Malformed, UnknownOpcode, WrongScope, WrongVersion, NonzeroReserved, WrongHandleKind |

Editor operation codes: Construct1 SetTool2 SetBrush3 Select4 BeginStroke5 Append6
CommitStroke7 BeginTransform8 UpdateTransform9 CommitTransform10 Cancel11 Delete12 Undo13
Redo14 New15 Replace16 Query17. No enum discriminant/order casts are permitted.

## Records and identifiers

C = session_ref:u64, document_session:u64, interaction:u64 (0 means none).
Brush = red,green,blue,alpha,width:f64 (40 bytes).
Resource = provider:u32, access:u32, resource:u64, canonical:u64, generation:u64 (32 bytes).
Access bits read1/write2/confirmed-write4. Capabilities bits edit1/export2.

Nonzero opaque transport IDs are allocated runtime-globally, monotonically and never reused;
exhaustion fails before mutation/transfer. Private registries hold actual facade handles and
issued effect tokens. Resolve session_ref, compare document/interaction values to the actual
projection, then reuse those typed values. Never reconstruct private handles from numbers.
Request is correlation, not permission or deduplication. Effect references are valid only
for their issuing session and current continuation. Zero denotes absence, never a live handle.

## Owner operations

0 is bootstrap only, 30/31 are transfer calls with owned sidecars, all others use owner call.

| Opcode | Exact payload | Success body |
| --- | --- | --- |
| 0 CreateApp | sessions,operations,destinations,buffers,buffer_bytes,document_bytes,save_jobs,snapshot_bytes:u64 | app through typed bootstrap |
| 1 CreateSession | capabilities:u32,tool:u32,Brush | Projection; ReuseBinding default |
| 8 CreateSessionWithSavePolicy | capabilities:u32,tool:u32,Brush,policy:u32,reserved:u32=0 | Projection; policy 0 ReuseBinding, 1 AlwaysChoose |
| 9 DocumentBounds | session_ref:u64 | document_session,revision:u64,present:u32,reserved:u32=0,x,y,width,height:f64 |
| 2 Detach | session_ref:u64 | empty |
| 3 Projection | session_ref:u64 | Projection |
| 4 Availability | session_ref:u64,Action | enabled:u32,reason:u32 |
| 5 Stats | empty | Stats |
| 6 SelectedBounds | session_ref:u64 | x,y,width,height:f64; existing core error when absent |
| 7 Diagnostic | empty | present:u32,reserved:u32,[Diagnostic] |
| 10 Edit | C,Edit | Outcome |
| 11 Action | C,Action | Outcome |
| 12 Input | C,pointer:u64,count:u32,reserved:u32,Sample[count] | Outcome |
| 20 File | C,tag:u32,reserved:u32 | Outcome |
| 21 Reply | C,effect_ref:u64,decision:u32,reserved:u32 | Outcome |
| 22 Resource | C,effect_ref:u64,Resource | Outcome |
| 30 TakeWork | empty | present:u32,reserved:u32,[WorkInfo]; unique Work sidecar |
| 31 Complete | empty; unique Result sidecar | empty |
| 32 Apply | session_ref:u64,operation_ref:u64 | Outcome |
| 33 Cancel | operation_ref:u64 | empty |
| 34 Release | empty | present:u32,reserved:u32,[release_ref:u64,Resource] |
| 40 Scene | session_ref:u64,previous_scene_ref:u64,Viewport | SceneInfo |
| 41 Descriptors | scene_ref:u64,first:u32,limit:u32 | count:u32,reserved:u32,Stroke[count] |
| 42 GeometryInfo | scene_ref:u64,geometry_ref:u64 | GeometryInfo |
| 43 GeometryCommands | scene_ref:u64,geometry_ref:u64,first:u32,limit:u32 | count:u32,reserved:u32,Path[count] |
| 44 ReleaseScene | scene_ref:u64 | empty |
| 50 ViewSource | C,source_generation:u64,Resource | Outcome |

File: Save1 SaveAs2 ExportCopy3 New4 Open5 Close6; Checkpoint7/Recover8 reserved Unsupported.
Reply: Cancel1 Save2 Discard3 Continue4 ConfirmClose5. Source/destination use opcode22 only.
Availability reasons: Enabled0 ReadOnly1 InteractionActive2 WrongTool3 NothingSelected4
HistoryEmpty5 NoActiveStroke6 NoActiveTransform7. Explicit matches, not casts.

Edit/Action prefix: tag:u32,reserved:u32. Exact variant payload follows:

| Edit | Payload |
| --- | --- |
| SetTool1 | tool:u32,reserved:u32 |
| SetBrush2 | Brush |
| SelectAt3 | x,y,tolerance:f64 |
| SelectCycle4 | direction:i32,reserved:u32 |
| ClearSelection5 | empty |
| BeginStroke6 | pointer:u64,device:u32,reserved:u32 |
| CommitStroke7 | pointer:u64 |
| BeginTransform8 | empty |
| UpdateTransform9 | scale,anchor_x,anchor_y:f64 |
| CommitTransform10,CancelInteraction11,Delete12,Undo13,Redo14 | empty |

Actions: ChooseTool1(tool:u32,reserved:u32), Cycle2(direction:u32,reserved:u32),
Delete3/Undo4/Redo5(empty), Scale6(anchor:u32,reserved:u32,factor:f64). Anchor 1 is the committed selection
center; additive anchor 2 is its committed transformed top-left. Both action execution
and availability accept these tags; other tags and nonzero reserved fields are rejected.
The 24-byte action body and opcode 11 remain unchanged. The literal
`action-scale-top-left.bin` fixture adds tag 2 without changing existing fixtures.
Tool Pen1/Select2; cycle Previous1/Next2; device Mouse1/Pen2/Touch3.
Sample: kind:u32,flags:u32,id:u64,x,y,timestamp,pressure:f64 (48 bytes).
Kinds Actual0/Predicted1/Correction2. Flag1 means pressure present, otherwise pressure zero.

## Results

Projection (152 bytes): session_ref,document_session,revision,scene_generation,interaction,
stroke_count,selected_id,canonical_destination,binding_generation:u64;
tool,interaction_kind,capabilities,state_flags,file_phase,navigation,outstanding_saves,reserved:u32;
Brush; continuation_ref:u64. Interaction Idle0/Drawing1/Transforming2; state bits dirty1,
undo2,redo4,binding-pending8. File Idle0/Destination1/Source2/Unsaved3/Writes4/Replacement5/
CloseReady6. Navigation None0/New1/Open2/Close3/Recover4. Destination is correlation only.

Outcome: Projection,changes:u32,invalidation:u32,Effect,AppliedSave. Changes document1,
session-replaced2,settings4,selection8,interaction16; invalidation None0/Overlay1/Full2.
Effect: tag:u32,argument:u32,effect_ref:u64,operation_ref:u64,revision:u64,Failure (72 bytes).
Tags None0/ChooseDestination1/ChooseSource2/ConfirmUnsaved3/AwaitWrites4/WorkQueued5/
NavigationReady6/CloseConfirmed7/Cancelled8/Failure9. Unused fields zero. Destination
argument purpose Save1/SaveAs2/Checkpoint3/ExportCopy4.
AppliedSave (72 bytes): present,purpose,result,ack,durability,write_failure:u32;
captured_session,captured_revision,destination,sequence,binding_generation,original_request:u64.
SaveResult Written1/Uncertain2/Failed3/CancelledBeforeStart4/WrittenAfterCancellation5/
FailedAfterCancellation6/UncertainAfterCancellation7/DispatchedUnconfirmed8/DispatchedAfterCancellation9.
Ack SavedCurrent1/SavedOlder2/CopyWritten3/CheckpointWritten4/StaleSession5/StaleBinding6/
Superseded7/Cancelled8/Failed9/CopyDispatched10. Durability FileAndDirectorySynced1/
DirectorySyncUnconfirmed2/ProviderConfirmed3; absent fields zero.

Stats: sessions,operations,resource_leases,buffers,buffer_bytes,outstanding_saves,
diagnostic_drops,transport_buffers,transport_bytes,scenes,scene_bytes,geometry_entries:u64.
Diagnostic: app,source_operation,session_ref,request:u64; provider,kind,result,durability,
write_failure,reserved:u32; Failure (96 bytes). source_operation is facade correlation,
not an executable operation_ref. Kinds Completed1/Write2/ReplacementFailed3/Cancelled4/Rejected5.
For kind2, result is SaveResult; for kind5, result is the explicit WireError status; all other
kinds have result0. Durability/write_failure are nonzero only for kind2; Failure carries
structured context for kinds3/5. Flat rejected Capacity/Busy/Unsupported remain distinguishable.

## Worker lane and ownership

WorkInfo (72 bytes): operation_ref,session_ref,request:u64; kind,resource_present:u32;
Resource;maximum_read_bytes:u64. Kinds Write1/Read2/New3/Retire4. Absent resource is zero.

| Opcode | Payload | Result |
| --- | --- | --- |
| 100 Info | empty | WorkInfo |
| 101 Encode | empty | result_present:u32,reserved:u32,length:u64; optional Result sidecar |
| 102 PrepareRead | empty; owned bounded bytes sidecar | Result sidecar |
| 103 PrepareNew | empty | Result sidecar |
| 104 FinishWrite | outcome:u32,detail:u32 | Result sidecar |
| 105 FailBeforeIO | failure:u32,reserved:u32 | Result sidecar |
| 106 Dispose | empty | Result sidecar |
| 107 CopyEncoded | offset:u64,maximum:u32,reserved:u32 | byte_count:u32,reserved:u32,bytes |

Finish Written1(detail durability),Failed2/Uncertain3(detail write failure),Dispatched4(detail0).
Invalid outcome retains encoded work. Encode is once-only; failure consumes work into its
actual issued failure Result. Complete cannot be called using caller-authored numbers alone.
Transfer calls reserve reply/result capacity and identity before consuming anything. Input
bytes for PrepareRead move only on accepted work; invalid scope retains work/input. Oversize
or corrupt content produces an issued failure, never partial installation. Copy <=64KiB.

WireApp stays !Send/!Sync. Work moves to one worker, Result returns to owner, never concurrent.
No filesystem/thread/renderer is started here. Future native FFI enforces actual owner thread,
catches unwind, uses nullable opaque pointers and nulls only consumed handles. A panic cannot
fabricate a successful receipt or unblock uncertain in-flight work; poisoned/stalled teardown
is explicit. Wasm-v2 uses bounded nonreused u32 object registries, while wire IDs stay u64.
JS uses BigInt; reacquire and copy views after allocation/memory.grow and before await. Browser
Worker/CanvasKit placement is a future capability check, not assumed OffscreenCanvas support.

## Scene records and budgets

Viewport: x,y,width,height,margin:f64. Stroke (48 bytes): stroke_id,order,geometry_ref:u64;
scale,tx,ty:f64. GeometryInfo: origin_x,origin_y,bounds_x,bounds_y,bounds_width,bounds_height:f64;
command_count:u64 (56 bytes). Path: verb:u32,reserved:u32,rgba[4]:f64,points[6]:f64 (88 bytes).
Verbs Move0/Line1/Cubic2/Close3. Preserve full f64, subtract origin before renderer narrowing.
SceneInfo (216 bytes): scene_ref,document_session,revision,generation,root_ref,committed_count:u64;
unchanged,live_present,preview_present,preview_replacement_present:u32;Viewport;
preview_stroke_id,preview_order:u64;live:Stroke;preview_replacement:Stroke.
Absent optional records zero; a culled preview still suppresses its original stroke.

Scene/geometry/root IDs are assigned opaque numbers; private Arc identity may find matches,
but pointer values never leave Rust. Previous same-key/same-viewport scene reuses its lease.
Unchanged committed root skips descriptor/geometry transfer; live path remains one fill for
actual+prediction. Geometry requests require membership of a live scene. Paging never flattens
all geometry on each input. Query failure preserves old scene and editor.

Transport defaults: 1MiB control packet, 64KiB copied/page payload, 64 buffers/128MiB total,
8 scene leases/128MiB unique pinned payload, 65536 geometry identities. Per-cache retained32MiB/
prepared64MiB; app cache budget128MiB. Reserve before publication, account outstanding worker
and scene leases, evict only disposable cache entries. Facade snapshots/buffers, transfer buffers,
cache and pinned scene budgets are separate conservative bounds, not measured process memory.

## Sequence and evidence

First freeze this codec/owner/work contract with exact binary fixtures under sketch-app-wire.
Then add bounded scene export and test unchanged-root reuse. Only after review add C/JNI/Wasm
wrappers/hosts. Linux may migrate directly to sketch-app now, with GLib owner, typed commands,
issued provider work and Skia separately; preserve current .05–32 zoom and native controls.
No host depends on finishing every ABI. Actual Linux display/portal/stylus remains a host gap.

Fixtures cover every request tag, error code, output enum, high-bit IDs and sample variant.
Tests include tails/overflow/reserved flags, wrong scope/owner, output-capacity atomicity,
real worker exchange, failed/cancelled receipts, aliases, stale continuation repair, independent
scene leases/eviction/unchanged geometry, and all facade tests. Codec fixtures may contain
synthetic wire IDs; execution tests must obtain real private ownership. No native acceptance
or performance claim follows from codec, replay, or compilation.

## Concrete typed implementation and host-wrapper obligations

`WireApp::bootstrap(&[u8], TransportLimits)` accepts opcode0 only. Bootstrap validates
sessions/operations/buffers 1..1024, destinations 1..2048, document bytes 24..64MiB,
buffer bytes at least document bytes and at most 512MiB, save jobs 1..1024 and snapshot
bytes 1..512MiB. Transport limits separately permit buffers 1..1024, bytes 0..512MiB,
scenes 1..64, pinned scene bytes 0..512MiB, geometry identities 1..65536. A zero byte
budget intentionally admits no allocations. `decode` is pure and cannot mutate an app.
One static checked allocator covers every WireApp and every transport handle kind in the
loaded process/Wasm instance. IDs from another app cannot accidentally resolve locally.

`WireApp::call(&[u8]) -> Result<Reply, WireError>` owns no caller buffer. Reply owns its
immutable bytes and a transport reservation until Rust Drop; `Reply::bytes()` borrows only.
There is no wire free opcode for replies: future C/JNI/Wasm wrappers must provide their
own exactly-once object free operation. No view may outlive its Reply. Failure packets are
fixed 72-byte stack values from `WireError::packet(opcode,request)`, requiring no registry slot.
The owner reserves a 4096-byte control reply before any action/event/resource consumption;
page replies reserve their exact bounded maximum. This is conservative capacity accounting,
not a claim that each small reply contains 4096 useful bytes. Stats include their own reply.

`take_work(request30) -> (Reply, Option<WireWork>)` transfers actual issued work. Calls to
30/31 through ordinary call are WrongScope. `complete(request31, &mut Option<WireResult>)`
consumes only on success; wrong app, phase, output capacity or ownership preserves the result.
The facade operation slot bounds one active work/result chain; result identity is reserved
before consuming work. A completed worker object has no work left and cannot execute again.
Ordinary controls/effects and work have separate private registries; no raw integer constructor
can fabricate proof. Dropping work/result is not completion and can leave shutdown busy.
`shutdown_ready()` also requires all response/input buffers and scene leases released.

`WireWork::copy_input(&[u8])` copies at most maximum_read_bytes+1 into owned InputBytes,
reserving the same app transport budget before allocation. Inputs are not document validation
proof. `WireWork::call(packet,&mut Option<InputBytes>) -> WorkerReply` retains input on
wrong scope, wrong app or rejected request; successful PrepareRead consumes it and holds its
reservation through decoding. Its Vec is not exposed. Corrupt/oversized content is an issued
failure result. A host must bound its original read as well; Rust cannot account an arbitrary
host allocation made before this copy. Result and encoded snapshot budgets remain distinct.

| State | Allowed operations | Ownership transition |
| --- | --- | --- |
| Issued Write | 100,101,105 | Encode -> Encoded or failure Result; FailBeforeIO -> Result |
| Issued Read | 100,102,105 | PrepareRead or FailBeforeIO -> Result |
| Issued New | 100,103 | PrepareNew -> Result |
| Issued Retire | 100,106 | Dispose -> Result |
| Encoded Write | 100,104,107 | Finish -> Result; invalid outcome retains Encoded |
| Consumed | none | InvalidState, no new proof |

105 is valid only before provider side effects. After Encode use104 with actual Written,
Failed (replacement definitely did not happen), or Uncertain outcome. The protocol cannot
verify a provider's truthful report; its adapter owns that guarantee. Never use FailBeforeIO
to guess that an interrupted write did not happen. Non-PrepareRead calls reject extra inputs.
Work is Send and serially movable; application ownership remains !Send/!Sync. Future native
wrappers must enforce thread identity and catch unwinding; this safe Rust crate exports no FFI.

## Scene consumption details

A scene request borrows `previous_scene_ref`; it never consumes or retains that lease.
Same key/viewport returns the same ref with unchanged=1 and **does not add a retain**.
For a changed frame: request using the old ref, retain the old view on failure, and only after
success with a different ref release the old ref once. Same ref must not be released merely
because another response reported it. `root_ref` equality independently signals unchanged
committed descriptors; live/preview portions can differ while root_ref stays equal.

Page first is a zero-based record index. first==count is EOF with zero records; first>count
is InvalidArgument. limit==0 returns zero records. Larger limits clamp to 1365 Stroke records
or 744 Path records, respectively, so record bytes are <=64KiB. An 8-byte count/reserved prefix,
48-byte status prefix and 24-byte frame header are additional. CopyEncoded likewise permits
65536 copied bytes plus these prefixes; offset==length is EOF, offset>length rejects.
A geometry ref must belong to the specified live scene, even if another scene retains it.
ReleaseScene is exactly once; a duplicate release is Stale. Scene commands are read-only.

Geometry path points use document-local stroke coordinates before the Stroke transform:
Move/Line use points[0..2], Cubic uses control1/control2/end in all six slots, Close uses none;
unused slots are zero. Reassemble **all** pages in order, preserving contours, and use one
nonzero-winding fill for the whole geometry. RGBA is normalized, straight alpha; retain the
existing Skia Color4f/no-explicit-source-color-space interpretation. This wire adds no color
management conversion. Do not fill each segment/page separately or multiply alpha twice.
For GPU narrowing, subtract GeometryInfo.origin in f64 first. With camera origin C, zoom Z,
stroke scale S and translation T, view = Z*S*(point-origin) + Z*(S*origin+T-C), then host
backing scale. Viewport is document space; this layer adds no desktop zoom clamp. Native
style/overlay geometry is outside this document/scene protocol.

One bounded SceneCache is reused across sessions, retaining at most 32MiB geometry and a
64MiB prepared root (below the 128MiB app cache allowance); switching sessions may evict it.
Live leases pin immutable data independently. Root and geometry reference counts charge each
unique payload once. Reusing a committed root updates constant-size registry state and only
examines new live/preview geometry; it does not flatten or walk all committed paths per input.
New roots require descriptor traversal. Geometry IDs resolve through a bounded direct index;
per-scene membership uses a sorted committed-root index shared across live frames, with
binary search plus at most two live/preview checks. Exporting N geometries therefore does
not rescan N strokes for every geometry/page. Both ID maps share the geometry_entries bound. Metadata identities and staged lookup work are bounded
by geometry_entries and MAX_STROKES; allocator overhead is excluded from byte estimates.
Query capacity/error preserves old leases; cache eviction cannot invalidate pinned scenes.
Released, unleased cached geometry may receive a fresh opaque identity on later publication.

## Test mapping for this slice

| Existing acceptance/invariant scope | Transport evidence |
| --- | --- |
| FFI-001, INPUT-001/002 | Exact fixtures, every truncation/tail, reserved/tag/float/count rejection, high-bit IDs, whole-batch preservation |
| FFI-001, ASYNC-001 | Two-app references/results/input rejection, actual worker thread exchange, one completion, cancellation and shutdown |
| SAVE-001/003, LOAD-001 | Alias releases, encoded outcome correction, output-capacity ownership, stale-load retirement and usable subsequent intent |
| RENDER-001, CACHE-001 | Full response/path goldens, paged reassembly, scene/root reuse, pinned capacity, old-scene survival and membership |
| Existing portable facade acceptance | Run the original 26 sketch-app tests plus issued-classification regression alongside transport tests |

The literal fixture set includes complete Projection/Outcome/WorkInfo/SceneInfo/Path responses,
not only tag units; payload handles and sample/resource IDs exercise values above 2^53 and
with the high bit set. These are transport/core evidence, never native input or GPU evidence.

## Additive save policy and whole-document bounds

Opcode 8 has a 56-byte body and returns the existing 152-byte Projection. Policy tags are
explicit; unknown values/reserved/tails fail before any session is created. Opcode 1 keeps
its original 48-byte body and ReuseBinding semantics. The typed policy is specified in
[save policy and bounds](022-save-policy-and-bounds.md); transport does not override it.

Opcode 9 has an 8-byte body and returns 56 body bytes. Present 0 means an empty document and
requires all four floating-point fields to be positive zero; present 1 carries the full
f64 committed outline bounds. Document identity/revision are captured on the same owner
call as the query. Hosts must compare both before applying a delayed Fit response.
This explicit O(committed samples) query excludes temporary/predicted input and viewport
culling; it does not construct a scene or cancel interaction. Platform zoom/padding remain
host presentation policy. Foreign/detached sessions fail; output reservation failure leaves
state and previously leased scenes intact. No existing opcode or ABI is reinterpreted.
