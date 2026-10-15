# Frozen extraction contracts

Task 015, starting revision `e467d402c0f9888ae11a5312a2caabcd9335480f` (runtime unchanged
from `codex/github-pages@712059d5`). This specifies target Rust boundaries for tasks
018/021; it does **not** add symbols to ABI 2 or Wasm protocol 1. Existing callers stay
on their current adapters until separately tested migration. [Compatibility](compatibility.md)
records current behavior and executable fixtures; [runtime](runtime.md) owns scheduling.

## Types, ownership and dependency direction

Each row identifies a cohesive file to create during extraction, not files already present.
Start in `crates/sketch-core/src/`; bridges depend on core, never the reverse. Files and
behavior tests stay <=200 physical lines. Split by responsibility if necessary.

| File | Types and responsibility | Dependencies |
| --- | --- | --- |
| `model/document.rs` | `Document`, `Stroke`, stable `StrokeId`, actual samples, brush, transform | Validated value types only |
| `model/sample.rs` | `SampleId`, `Sample`, `Pressure`, `DeviceKind`, validation | No platform event types |
| `model/style.rs` | `BrushSettings`, `Rgba`, `StrokeWidth` | No widgets/renderer |
| `model/transform.rs` | Uniform `StrokeTransform`, `DocumentPoint`, `Bounds` | Numeric validation |
| `application/ids.rs` | Distinct `DocumentSessionId`, `Revision`, `RequestId`, `ViewId` | Integer newtypes, never f64 |
| `application/commands.rs` | `EditorCommand`, `CommandContext`, `EditorError` | Domain types and session IDs |
| `application/queries.rs` | `EditorQuery`, `QueryResult`, `EditorViewState`, `Availability` | Immutable projections |
| `application/session.rs` | `EditorSession`, one mutation owner, dispatch | Document, interaction, history |
| `application/effects.rs` | Owned `Effect`, `SaveToken`, `Completion` | Immutable snapshot and correlation |
| `application/capabilities.rs` | `Capabilities { edit_document, export_document }`, checks | Explicit session construction |
| `interaction/controller.rs` | `ActiveInteraction = Idle / Drawing(InkSession) / Transforming(TransformSession)` | State transitions |
| `interaction/ink.rs` | Actual draft and replaceable predictions; captured brush/device/pointer | Validated input batches |
| `interaction/transform.rs` | Original transform, preview, anchor; commit/cancel | Document geometry |
| `interaction/batch.rs` | Borrowed `SampleBatch`, validated batch plan | Sample identity/order rules |
| `history/mod.rs` | `UndoHistory`, `DocumentChange = Insert / Delete / Transform` | Committed document changes |
| `geometry/{outline,bounds,hit_test}.rs` | Pure geometry operations | Immutable model |
| `viewport.rs` | `Viewport`, `ViewPoint`, map/inverse, explicit host zoom/resize policy | No document mutation |
| `scene/{snapshot,builder}.rs` | `SceneSnapshot`, `SceneKey`, `RenderInvalidation`, preview overlay | Read-only document/session |
| `persistence/{codec,reader,validation}.rs` | Stable format, bounds, fully validated replacement | Model only |
| `persistence/snapshot.rs` | `DocumentSnapshot`, immutable owned worker input | Model and revision identity |
| `source_policy/` | Existing identity/version policy, independent of transport | No network/watch objects |

`sketch-ffi/src/{types,handles,status,input,document,scene,source}.rs` owns C layouts,
serialization, panic containment and conversions. `sketch-wasm/src/{registry,buffers,
reader,dispatch,encoding}.rs` owns protocol tokens and bytes. Native file I/O leaves the
codec; preserve synchronous compatibility APIs while introducing the storage coordinator.
Tests use behavior-named integration files rather than extensions to oversized `lib.rs`.

`Document` stores no selection/history/dirty flag or viewport. `EditorSession` owns those
session concerns and one active interaction. Each canvas owns its `Viewport`; native hosts
own text in progress, focus, accessibility and OS resources. Renderers own disposable
caches; neither cache nor renderer can acquire a mutable document reference.

## Command and query surface

These signatures are the agreed Rust API shape, not an ABI header. Constructors validate
untrusted values. Dispatch rechecks transitions/capabilities; UI availability is advisory.

```rust
fn execute(&mut self, context: CommandContext, command: EditorCommand)
    -> Result<CommandOutcome, EditorError>;
fn query(&self, query: EditorQuery) -> Result<QueryResult, EditorError>;
fn append(&mut self, context: CommandContext, batch: SampleBatch<'_>)
    -> Result<CommandOutcome, EditorError>;
fn complete(&mut self, completion: Completion) -> Result<CommandOutcome, EditorError>;
```

`CommandContext` carries session identity and request correlation. It does not grant
privilege. Future agent clients use this typed boundary; transport/auth is not included.

| Boundary | Concrete variants | Semantics |
| --- | --- | --- |
| `ToolId` | `Pen`, `Select` | Stable semantic IDs, native labels/icons |
| `EditorCommand` session | `SetTool(ToolId)`, `SetBrush(BrushSettings)`, `SelectAt(point,tolerance)`, `SelectCycle(direction)`, `ClearSelection` | No history; reject illegal active transition |
| `EditorCommand` interaction | `BeginStroke(pointer,device)`, `CommitStroke(pointer)`, `BeginTransform`, `UpdateTransform(scale,anchor)`, `CommitTransform`, `CancelInteraction` | Begin captures brush; successful commit creates at most one history entry |
| `EditorCommand` document | `DeleteSelection`, `Undo`, `Redo`, `NewDocument`, `ReplaceDocument(ValidatedDocument)` | Requires edit capability; new/import reset history and session |
| `EditorCommand` service | `RequestSave(request)`, `RequestExportCopy(request)` | Return immutable owned snapshot effect, no synchronous I/O |
| `EditorQuery` | `ViewState`, `SceneSnapshot(view)`, `CommittedSnapshot` | Immutable output; do not clear dirty or advance revisions |
| `QueryResult` | `ViewState(EditorViewState)`, `Scene(SceneSnapshot)`, `Document(DocumentSnapshot)` | Typed matching result, no native widget pointers |
| `Effect` | `WriteSnapshot { token, snapshot }`, `ExportCopy { request, snapshot }` | Host resolves destination/provider; copies never acknowledge save |
| `Completion` | `SaveSucceeded(SaveToken)`, `SaveFailed(SaveToken, FailureKind)` | Owner thread validates identity before state update |

`EditorViewState` contains session/revision, tool/brush, selected ID/bounds, active interaction,
undo/redo availability, dirty and capabilities. Disabled reasons use semantic keys such as
`interaction_active`, `read_only`, `nothing_selected`, `history_empty`; native hosts localize.
Do not move native default colors/widths into shared defaults without recording their current
differences. Preserve current brush validation: normalized finite RGBA and width 0.01–10000.

`CommandOutcome` contains a new small view projection, typed `StateChanges`, a
`RenderInvalidation`, and owned effects. Failure returns `EditorError` and no effects or
partial state changes. Errors distinguish invalid value, invalid transition, read-only,
stale session/completion, resource limit, unsupported/corrupt document and service failure;
existing adapters keep their current numeric status mappings until a versioned extension.
Errors include operation/field/category and correlation, never input points, file paths or
content. Do not silently map a new behavioral failure into an old successful operation.

Read-only viewers allow state/scene/document queries, selection and viewport navigation.
They reject stroke/transform/delete/history/new/user-import commands in Rust. An explicit
`replace_viewer_snapshot(ValidatedDocument, SourceToken)` service boundary may replace the
selected viewer source after identity checks; it is not exposed as an edit command. Export
and save availability depends on distinct export/edit capabilities. Agent permissions are
deferred; this capability boundary must not be presented as agent authentication.

## Samples, transactions and views

`SampleBatch<'a> { pointer: PointerId, samples: &'a [Input] }` is borrowed for one call and
never retained. Input enum remains `Actual / Predicted / Correction`. Normalization preserves
optional pressure and timestamp/sample identity. Current core has no tilt/buttons/clock-domain
fields; adapters retain richer native metadata separately until an intentional schema extension.
A native clock is converted once to monotonic seconds; different clocks are never mixed in one
stroke. Pointer identity is stable until commit/cancel. IDs are integers across bridges.

Validate all samples and resulting limits before modifying actual **or predicted** state.
Actual IDs strictly increase and timestamps do not decrease. Actual samples cannot follow
predictions in a batch. Corrections must refer to an actual sample already present before
this batch, with identical timestamp; correction of an earlier item in the same batch is
currently rejected. Predictions replace the previous prediction list on each successful
append, including an empty batch. Predictions never enter history/serialization. Commit
requires at least one actual sample; cancel changes neither committed bytes nor history.

Canonical view mapping is `view = document * zoom + translation`, inverse
`document = (view - translation) / zoom`, in native logical points; pixel scale is a separate
surface property. Anchored zoom keeps the mapped document point under the same view point.
Resize keeps the document center fixed. Cancellation precedes a remap. Host policy selects
zoom limits and diagonal projection versus radial transform gestures during extraction;
[compatibility](compatibility.md) prohibits accidental unification. Uniform stroke scaling
is absolute relative to gesture begin and includes brush width; updates replace preview.

## Snapshot, rendering and save lifetime

`SceneKey { session, revision, interaction_generation }` identifies document plus transient
state. `RenderInvalidation = None / Overlay / Strokes(ids) / Full` is an invalidation hint,
never permission to discard ordered document changes. First extraction uses full immutable
snapshots; incremental deltas/caches are task 020. `SceneSnapshot` owns its data and is
renderer-neutral. `ViewGeneration` and `SurfaceGeneration` are separately captured by a
render job and checked before presentation; an old preview cannot overwrite newer ink.

`SaveToken { document_session, snapshot_revision, request_id }` binds one immutable snapshot
to one save. Session identity changes on new/import and handle replacement; revisions are
not compared across sessions. Undo can return to a previously saved revision. Save success
may mark clean only if the current session/revision still match. Correlate failures too.
Task 021 serializes writes by destination identity so old writes cannot overwrite new ones;
stale acknowledgement rejection alone does not prevent that data loss. Cancellation is
best effort; completed writes are still processed using their token, never assumed undone.
Worker inputs are immutable owned snapshots. No render/storage worker mutates editor state.
The host posts completions onto the one editor owner. One-shot cancellation/close invalidates
pending callbacks before freeing bridge handles; no worker keeps borrowed FFI pointers.
