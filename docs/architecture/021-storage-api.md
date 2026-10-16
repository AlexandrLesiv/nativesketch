# Task 021A: portable save admission and acknowledgement

The implementation is additive under `sketch_core::application::storage`. It does not
create OS threads, write files, schedule checkpoints, or change the legacy native/Wasm
consumers. Task 021B supplies services; task 019 adopts the typed native bridge.

## Ownership and API

One process-wide `SaveCoordinator` serializes requests from editor owners. Each host assigns
canonical opaque `DestinationId` values and retains paths/access grants outside the core.
Use the same coordinator for all windows; separate coordinators do not serialize writes.
Hosts must coordinate simultaneous windows bound to the same file: FIFO writes alone do
not refresh another window's document or establish a file-watching/conflict policy.

1. `request(&mut EditorSession, RequestId, SavePurpose, DestinationId)` checks capability,
   idle state, destination binding, queue capacity and estimated snapshot payload memory.
   It reserves a global monotonic write sequence and memory before synchronously copying
   the exact committed revision. No asynchronous gap can silently change that revision.
2. `start_next()` returns the oldest queued `WriteJob`, only when no job is active.
   The host moves this unique owned input to its serial worker. It contains no editor borrow.
   `encode()` belongs on that worker; its bytes preserve the existing document format.
3. `WriteJob::complete(self, WriteOutcome)` drops its snapshot and returns a small,
   non-cloneable receipt. The host returns it to the coordinator owner for `finish()`.
4. `take_completion(token)` consumes the registered result once. The editor owner consumes
   the resulting non-cloneable `SaveCompletion` with `acknowledge_save()` and refreshes
   its O(1) state. No disk work, encoding, or document traversal occurs during acknowledgement.

`WriteJob` must produce a receipt even when the worker operation fails. The service must
supervise panics/shutdown and report an explicit outcome; dropping an active job does not
free its slot or permit another writer to overtake a potentially still-running operation.
No borrowed pointer or Rust object is newly exposed across FFI in this phase.

## Admission and cancellation

`StorageLimits` bounds outstanding jobs (including unconsumed completions) and aggregate
snapshot payload bytes. Accounting uses checked stroke/sample lengths, preserves the
64 MiB encoded-document ceiling, and excludes allocator bookkeeping and encoded worker
buffers. A single worker may additionally hold an encoded document of up to 64 MiB.
Configuration is not a process RSS guarantee; native services must budget their own buffers.

Explicit Save/Save As requires edit and export capabilities; Export Copy requires export;
internal Checkpoint requires edit. Read-only export is allowed when explicitly enabled.
Save requires the current bound destination. Save As reserves a binding change until its
completion is acknowledged; another Save As is rejected during that time. Checkpoint cannot
target the current bound destination. Export Copy/checkpoint cannot target the pending
Save As destination, preventing a newer copy from invalidating its deferred binding result.
Ordinary Save to the old binding remains available during Save As.

All new storage requests require an idle interaction. Native file actions cancel transient
input before requesting; checkpoint callbacks defer while an interaction is active. The
existing explicit `CommittedSnapshot` query retains its previous active-interaction behavior.
Queue-full requests fail visibly with `Busy`; this phase does not silently coalesce saves
or implement checkpoint priority/coalescing. Those scheduling policies belong to 021B.

Queued cancellation drops its snapshot and yields `CancelledBeforeStart`. Active
cancellation preserves its slot and waits for a receipt. A completed write after cancellation
is reported distinctly; cancellation never promises to roll back the destination.

## Saved-state semantics

Tokens include session, request, revision, destination, binding generation, write sequence
and purpose. Unknown/duplicate completions cannot be issued; stale-session completions cannot
mutate replacement sessions. New/import resets binding state with the new session identity.
Close/open navigation must still wait for outstanding explicit writes as the native contract
requires; callback rejection alone cannot undo a real write to an old destination.

Successful manual saves record the revision actually written. If B finishes after an undo
to previously saved A, A becomes dirty and redo to B becomes clean. Newer writes to the same
destination supersede older acknowledgement delivery. Successful Save As adopts its target
even if a later ordinary Save to the old destination was acknowledged first. Same-destination
Save As preserves the saved revision of a newer completed write.

Checkpoint/export-copy never mark clean. An export copy, cancelled completed write, uncertain
write, or obsolete-binding write that affects the current destination invalidates previous
clean knowledge where necessary. Writes known to fail before replacement preserve it.
`WriteOutcome::Failed` requires the adapter to establish that replacement did not occur;
`Uncertain` is required when it cannot establish the outcome. Directory-sync warnings after
successful replacement are written results, not unchanged-file failures.

The old synchronous `Editor::mark_saved` keeps its previous current-revision/idle checks.
Its ABI 2 and Wasm 1 callers are unchanged. The new history acknowledgement is private and
reachable only through the typed completion boundary.

## Cost and verification

Snapshot capture performs O(strokes) size preflight and O(document samples) copying in a
file-action or idle callback, never a pen callback. It may delay subsequently delivered input.
`cargo run --release -p sketch-core --example snapshot_cost` measures this synchronous copy
separately on synthetic 1k/10k/100k/1m-sample documents. It does not measure native input,
pen-to-screen latency, disk latency, real recovery, or platform UI responsiveness.

Tests under `application::storage::tests` cover SAVE-001/002/003, VIEW-001, INK-002 and
ASYNC-001; persistence estimate tests cover size ceilings/overflow before allocation.
Existing document, ABI and Wasm conformance fixtures remain authoritative. Actual worker
failure, process-kill, recovery-store and native UI evidence are pending subsequent phases.
