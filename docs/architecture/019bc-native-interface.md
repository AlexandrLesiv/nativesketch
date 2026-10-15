# Native interface v1: proposed header contract

Header/spec tranche for agreement at base `6567c4e`; declarations are not implementation
or native adoption evidence. New crate `sketch-native-ffi` will depend on core and services,
with host-scoped tests. Legacy `sketch-ffi`/ABI2 and Wasm1 remain unchanged. Swift's new
`CSketchSession` module includes only `sketch_session_v1.h`; its include graph exposes no
legacy functions. Production app, smoke journey and headless consumer use the same new path.

## Ownership and bounded runtime

One app-owned runtime creates one storage worker and owns a bounded session registry.
A unique opaque session handle owns one slot. Stable window identity differs from the
replaceable document-session identity. Every session call passes its runtime; wrong-runtime
or wrong-owner-thread use is rejected before mutation. All runtime/session/scene calls and
release occur on the creating UI owner. Inputs are borrowed for one call; asynchronous paths
copy bounded descriptors and transfer owned snapshots/tickets only. No worker sees C handles.

Constructors initialize outputs to null. Other rejected operations preserve output and
ownership, except documented count outputs. C pointer validity/length/unique ownership are
caller obligations; no function promises to validate arbitrary dangling pointers. Each entry
contains panic; mutation panic poisons affected state. Destruction remains available through
asynchronous detach/drain. No panic unwinds into Swift.

Defaults: 16 sessions, 8 operations, 128 reserved result/notification slots, 128MiB coordinator
snapshot payload. Zero numeric limits select those defaults; struct_size must equal the v1
config size, and build_id is an opaque diagnostic correlation (zero allowed). Nonempty roots
must be bounded absolute UTF-8 paths; empty roots explicitly disable their respective service
and expose unavailable state/counters. Opening, locking and creating roots run on the worker;
a path access/lock failure preserves the runtime and explicit document I/O in degraded mode.
Hard maxima: 32 sessions, 32 operations, 256 events. Validate configurations
that cannot reserve result/grant-release/retirement credit before accepting work. Pending
results retain their slot until applied/released. Only one immutable scene per session may
be outstanding; scene memory remains document-dependent and explicit, not an ink-path query.
Input batches max4096 samples, locations max16KiB UTF-8, display metadata max256 bytes.
Do not silently split/drop actual samples. Existing document/temporary buffer costs remain.

The session opaque handle identifies runtime-owned ApplicationSession state. Normal detach
requires CloseConfirmed; a pristine never-edited initialization or poisoned session also
permits detach so construction failure can unwind without opening a window. Accepted detach transfers state
to bounded retirement or a receipt-draining tombstone, nulls the handle, and never drops large
documents on the UI owner. Busy retains it for retry. Outstanding grants/receipts outlive the
window. Runtime shutdown is requested only after every attached session has CloseConfirmed.
It stops new admission but permits detach, apply/release and retirement while draining. Free returns Busy until
sessions are detached, issued events settled and worker stopped; it does not join on UI.

## Commands, effects and native projection

The command tags and DTOs in the versioned headers map explicitly to portable typed commands;
never cast Rust enum order into this ABI. File actions, controls, shortcuts and later agent
adapters share the dispatcher. Native focus/text validation precedes explicit file/navigation
commands: commit accepted SetBrush first, or retain invalid text and block the intent.

Projection is authoritative for tools, brush, capabilities, interaction, history, dirty state,
binding, continuation and outstanding count. Native presentation keeps incomplete field text,
selection/caret and undo. Completion never resets field content/focus automatically. Selected
bounds and owned scene snapshots are separate explicit queries. Scene count/copy refer to
one immutable generation; capacity failure never writes partial geometry.

Status != OK is rejection. Status OK may report an accepted failure effect or asynchronous
preparation. Failure domain/code plus correlation are redacted; the returned projection tells
whether a continuation remains actionable. Save outcomes expose actual captured/written
revision, destination, acknowledgement and durability. Native retry is a fresh typed intent,
not replay of an old success. ConfirmClose is followed by immediate synchronous native close;
app-wide terminateLater receives exactly one native reply.

## Ordered resolution, admission and storage

The first slice supports trusted local filesystem locations, with the reviewed std-only
symlink/race/ACL limitations. Unsupported providers return an explicit error. Native code owns
security-scoped access objects and supplies unique grant IDs; Rust owns descriptor/binding/job
references. Release occurs exactly once through GRANT_RELEASE after all references settle,
including after window detach. Existing bindings retain grants for future Save operations.
A grant ID is nonzero and unique within a runtime; host retains its access object until
GRANT_RELEASE, even if the originating panel or window has closed.

SupplyLocation validates token/capacity and reserves an ordered operation barrier, copies the
descriptor, then resolves/validates the path on the worker. Existing parent canonical path
and target entry identify destinations; reconcile identity after replacement. Case and symlink aliases are resolved by canonicalizing an existing target where possible;
new targets use canonical parent plus entry name. Hard-link entries remain separate targets
because atomic replacement changes only that directory entry. Global serial I/O is the
final ordering guarantee; provider aliases beyond this local policy are unsupported. Native
platform provider/coordination support is not implied by an absolute path.

Resolution readiness produces a retained owner event. Applying it invokes the SAME typed
destination/source continuation. Automatic readiness MUST defer while a new interaction is
active; it cannot turn a background event into a fresh intent that cancels new ink. Commit
or explicit cancel schedules a later owner idle turn to retry, never snapshot copy inside the
pen callback. Cancellation remains available while the barrier waits. No later accepted I/O
can overtake it. A stale source authorization preserves current content.

Unknown destinations show Preparing. The exact snapshot is captured at resolved admission,
not at the original menu/chooser click; capture identity/revision then appears in outcome.
Bound destinations admit/capture immediately only when no unresolved runtime barrier exists.
While resolution/admission is pending, all later I/O intents across all windows return Busy
before dispatcher mutation or draft cancellation. Availability reports the same guard.
This prevents later bound saves entering the core FIFO ahead of unresolved destinations. Save/Open share ordered barriers so a read
cannot overtake an accepted write. All encoding/validation/read/write/flush and candidate
preparation run on the worker. Candidate installation reserves retirement credit first;
rejected candidates and trusted failures/completions remain owned until correct delivery.

Poll returns at most32 metadata notifications and never captures/decodes. Issued payloads
stay in bounded runtime storage until ApplyEvent/ReleaseEvent. Attached save results cannot
be discarded through ReleaseEvent. Metadata is emitted once, not on every poll. ApplyEvent
OK consumes the event including accepted failure; Busy/Stale retain it. Caller releases a
stale/cancelled read or resolution event: runtime cancels only its still-matching continuation,
queues owned candidate destruction off-owner, clears its barrier and releases its grant once.
Cancelling a pending SupplyLocation similarly marks the operation cancelled; eventual result
is discarded off-owner and a terminal notification is released, never applied to another intent.
A rejected release preserves ownership. Cancel never authorizes dropping a save receipt. Detached results settle internally against their original
session/tombstone and never open obsolete UI. One app-owned callback/timer runs while work is
pending; otherwise use a checkpoint deadline and stop polling. Idle performs at most one due
capture after checking interaction/explicit-write priority. Timers are scheduling targets.

## Recovery, diagnostics and deterministic tests

Host secure randomness supplies a fresh128-bit recovery identity for each document session.
Registration copies identity/name; private store allocation happens asynchronously.
Worker holds an OS-backed exclusive File::try_lock on the recovery root lock file for its
lifetime. A competing runtime receives observable unavailable recovery; a crash releases
the lock without stale-lock deletion. Diagnostics/recovery failure degrades those services
without disabling drawing or silently losing accepted explicit save obligations. Replacement
requires registration for the new session before checkpointing. Native wrappers do this
immediately after the session-change outcome. Unregistered/failed recovery is visible, never
reported as active protection. Default2s/30s scheduling and reviewed retention/capacity rules
remain. Explicit saves take admission priority; failure suspends retries.

RecoveryList returns a bounded owned inventory event (max20 entries), opaque item IDs, exact
identity/generation/revision/time/name, rejected/fallback counts and known/unknown capacity
usage. Keep the event while its UI list is visible. Open captures the selected item tuple
before list release; the worker reads that exact generation or fails stale, never substitutes
new unseen content. It enters the same unsaved-navigation route and restores dirty/unbound.

Confirmed Discard validates the selected candidate identity/generation before bounded cleanup;
a newer valid generation causes conflict. No opening, Later or list dismissal resolves data.
After a confirmed explicit save, the runtime queues resolution of the exact associated
recovery identity/generation. Never resolve a newer checkpoint merely because its runtime
session matches. Resolution completion/failure is observable. Capacity preserves existing
recoveries and exposes management/retry; no hidden endless retry. RETRY uses item_id=0 and
stable window ID, or window=0 for all registered sessions. It reserves an asynchronous store
recheck: unavailable/capacity/suspended → preparing → active or explicit failure state. It
works without an inventory item and never invents a missing recovery identity.

Runtime structured diagnostics use bounded ring/rate/rotation services. Polling reports drop
and sink-failure counters. DiagnosticsExport copies only bounded redacted records to a
user-chosen local location through the ordered worker. It never exports document/name/path
content. No flush or diagnostic filesystem work occurs in input or poll callbacks.

DEBUG/test-feature-only worker gates/fail-next stages use an explicitly supplied disposable
control root. They hold/fail actual storage boundaries, preserving native command admission,
snapshot, encoding/write and trusted acknowledgement. Production has no test-control entry
points or environment-triggered bypass. Record injected timing separately from real disk
failures. Required cases include ink begun during resolution, no overtake, delayed save A
while drawing B, stale reads, grant release after detach, recovery resolution/discard/capacity,
shutdown drain, panic containment and the twelve agreed native journeys.

## Exact consumer behavior and acceptance

`sketch_tags_v1.h` freezes every shared discriminant and failure subcode. Zeroed reserved
fields are required; unknown enum tags and capability bits reject before mutation. Availability
accepts command tags except SetTool, which uses CHOOSE_PEN/CHOOSE_SELECT. CommitStroke uses
payload.stroke.pointer with device/reserved zero. No-selection SelectedBounds returns
NOT_FOUND and leaves output untouched. Other rejection leaves outputs untouched; constructors
set output null, while count/copy capacity failures set required count but copy no elements.
Scene handles retain an independent owner-thread/lifetime token, outlive their session/runtime
safely, and release their own geometry without dereferencing a freed runtime.

Outcome retry_action is Rust-selected, zero when not offered. Failed SaveAs returns SAVE_AS
and opens a fresh chooser; Save returns SAVE; ExportCopy returns EXPORT_COPY. Failed
navigation read abandons that continuation and offers OPEN (fresh chooser); recovery retry
requires selecting from a fresh inventory, so retry_action=0. A failed save during navigation
offers its original Save/SaveAs action as a fresh save only. Shared core clears the original
Close/New/Open navigation; the user must request it again. Cancellation never resurrects it. No stale/cancelled failure shows a sheet.
Preflight rejections return status only and are not retry offers. Post-replace uncertainty is
identified by save_result, separate from pre-replace failure and durability warning.

A frozen inventory event owns its item IDs. Copying items does not release it. Release once
after list dismissal or accepted Open/Discard has copied the selected exact tuple. Poll with
capacity0 emits nothing; out_count=0 and state reports ready count. wake_after_ms=UINT64_MAX
means no scheduled wake, zero means schedule an owner idle turn, never recurse into input.

Contract tests must include real compiled C layout/call consumer; invalid trailing sample
rejects the whole batch; new ink during destination/source resolution survives readiness;
two-window unresolved SaveAs rejects later bound Save and Open without cancellation;
a second runtime/process cannot own the same recovery root and remains usable when degraded; stale
read release permits the next Open and shutdown; grants release exactly once after detached
completion; held SaveA/editB and actual on-disk A; shutdown drain and wrong-thread/panic cases.
Native E2E additionally exercises the twelve cases in docs/design/019-native-command-adapter.md.
The test-only control root uses fixed `reached`/`release` marker names and one matching boundary;
Storage stages match explicit Save/SaveAs only, excluding checkpoints and diagnostics.
RESOLVE matches user source/destination work, excluding root/registration initialization.
External test releases the worker after observing reached. HOLD expires after60s or shutdown
request, returning a stage-appropriate injected failure/uncertainty instead of hanging.
Diagnostics remain redacted.
