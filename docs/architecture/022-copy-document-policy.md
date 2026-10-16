# Copy-document policy prerequisite

> Migration update (2026-10-16): follow [typed bindings migration](022-typed-bindings-migration.md).
> Its owner-selected typed FFI/generated declarations supersede custom packet/layout
> prescriptions here. Preserve semantic/provider/lifetime requirements; existing code and
> historical evidence are not automatically migrated or invalidated. Do not expand packet consumers.


Proposed shared contract after the lead, explorer and independent reviewer inspected
`df156a7`. Implement after the JNI transport checkpoint, with the Rust contract owner.
This is not implemented behavior or Android native acceptance. Final additive wire tags
and constructor layout must be frozen before parallel consumers start.

## Purpose and compatibility

Android imports an editable copy and saves through a newly chosen provider destination.
Neither completed operation retains a reusable file binding. `AlwaysChoose` changes
routing only, so it does not preserve these resource lifetimes by itself.

Introduce an immutable copy-document session mode, distinct from existing desktop binding
and chooser policies. Preserve it across New/Open replacements. A validated editable copy
import is clean and unbound. Save, Save As and guarded Save always choose a destination,
then issue a confirmed nonbinding save purpose, provisionally `SaveCopyConfirmed`.
Keep the actual issued token → snapshot → work → receipt → completion → Apply proof chain.
Do not substitute viewer replacement or unconfirmed `ExportCopy`.

UIKit's capture-before-picker staged export needs an additional continuation protocol;
this destination-before-write contract does not solve that separate requirement.

## Serialization and ownership

Exactly one mutating provider operation is admitted per copy session. Its obligation
spans destination choice, queued/issued work, active cancellation, worker completion,
the Ready state and validated owner Apply. Worker exit or completion receipt delivery
does not remove the obligation. Cancelling a navigation wait dialog does not remove it.
Timeout alone cannot prove termination. Detach retains the owner and resources until the
actual work and result obligations settle; no slot reuse may redirect an old proof.

Enforce admission in the shared layer as well as advisory availability. Destination replies,
direct save-purpose calls, ExportCopy and Checkpoint must not bypass the barrier or establish
a binding. For the initial Android mode, reject unsupported ExportCopy/Checkpoint purposes
before mutation; this adds no Android control or feature. Keep existing desktop behavior.

The authoritative barrier must include owner-held issued save tokens until Apply consumes
them. `SaveCoordinator::start_next`, coordinator outstanding counts and worker liveness are
insufficient: completed results can already have left that coordinator. Existing
`explicit_pending` excludes checkpoints and must not accidentally admit an alternate route.

Resources retain canonical identity, generation and access rights only while an issued
operation needs them. Release after actual termination and application/retirement, exactly
once. Rejected transfers retain their original ownership. Never put successful copy imports
or saves into `SaveState.destination` or `Slot.bound_resource`.

## Saved-state and navigation rules

Validate exact app, operation, issued token, session and current document identity before
any acknowledgment or invalidation. A current command context alone cannot validate an old
receipt. Wrong, foreign, replayed and old-document proofs have no state effect.

| Valid result | Saved-state effect |
| --- | --- |
| Noncancelled Written | Acknowledge only the issued snapshot revision; current revision is SavedCurrent, intervening edits are SavedOlder |
| Failed, CancelledBeforeStart, FailedAfterCancellation | Preserve earlier saved knowledge because the provider guarantees no replacement |
| Uncertain, UncertainAfterCancellation, WrittenAfterCancellation | Conservatively clear known-saved revision; never authorize guarded navigation |
| Unconfirmed dispatch | Reject as incompatible with confirmed-copy save; never mark saved |

Guarded navigation requires its matching save to acknowledge the current revision, not
merely report Written. Older success must reconfirm unsaved changes. Failure, uncertainty
and cancellation stop that continuation. New/Open replacement cannot inherit old proof.

Conservative invalidation may mark a document dirty even when an uncertain write targeted
a different copy. The lead accepts this narrow failure-safety change: provider uncertainty
must not be presented as reliable saved state. It is stricter than legacy Android behavior,
not exact failure parity. Without retained destination identity, do not promise cross-window,
cross-session or external-writer persistence tracking. Android currently has one editor.

## Required tests and implementation boundaries

1. Clean editable copy import is unbound; direct Save, Save As and guarded Save choose and
   issue confirmed-copy work. Mode survives replacement and existing desktop defaults do not change.
2. A second write fails before mutation in every phase, including Ready before Apply;
   destination replies and every alternate write purpose cannot bypass the barrier.
3. Current and older-revision success have distinct acknowledgment/navigation outcomes.
4. Guaranteed-no-write failure preserves clean knowledge; all uncertain/cancelled-written
   cases invalidate it, including a previously clean copy to a different destination.
5. Cancel, timeout, detached UI and cancelled navigation waits retain the live obligation.
   Old-document, stale, foreign and replayed proof cannot dirty or clean a replacement.
6. Read/write success, failure, staleness and cancellation release resources exactly once
   after real termination; no completed binding remains. Rejected transfer keeps ownership.
7. Existing desktop binding-generation, write ordering, cancellation and navigation tests pass.
8. Real C/Wasm consumers verify additive construction/policy packets and exact error behavior
   before Android consumes them. Then actual JVM/provider tests verify the host translation.

Suggested shared changes are routing policy/replacement, storage admission and acknowledgment,
facade resource capture/release and their tests. One Rust owner changes these contracts.
Keep files at most 200 physical lines and retain correlated failure diagnostics without
document bytes or provider URLs. Fresh review follows implementation and material fixes.

Android URI permission ownership remains separate: release only grants newly acquired and
owned by the operation; do not revoke a prior app grant or a grant shared by another request.
Actual SAF, recreation and lifecycle acceptance still require Android execution.
