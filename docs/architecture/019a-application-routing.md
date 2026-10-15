# Task 019A: portable application command routing

`sketch_core::application::routing::ApplicationSession` owns an `EditorSession`, one pending
native continuation, one semantic navigation intent, and bounded admitted-save obligations.
It constructs its editor from explicit capabilities, validated tool/brush and an optional
validated initial document; it cannot import an editor with hidden outstanding save jobs.
Existing `EditorSession`, ABI 2 and Wasm 1 consumers remain unchanged.

## Dispatcher and state

`dispatch(&mut SaveCoordinator, CommandContext, ApplicationCommand)` is the common route for
edit commands, borrowed sample batches, file intents, native effect replies, trusted save
completions, prepared replacement and issued read failure. The process-wide coordinator is
an explicit dependency, not a global service locator. There is no source-specific shortcut,
control or future-agent behavior. Context is correlation/stale-input validation, not agent
transport authentication or filesystem authorization.

Edit/input routes retain existing whole-batch validation, prediction isolation, brush capture
and transaction semantics. Wrapped low-level `NewDocument`/`ReplaceDocument` are rejected;
replacement must pass semantic navigation. Incomplete native field text never enters this
API. Validated brush commits use `Edit(SetBrush)`; canvas input reads the shared projection.

`projection` is O(1): editor state, binding state, file phase, issued continuation token,
navigation intent and outstanding-save count. Existing edit availability comes from the
editor projection; `file_availability` supplies capability/busy reasons using the same
preflight invoked by dispatch. Geometry and committed snapshots remain explicit queries.

`ApplicationError` reports rejection with context and leaves editor/pending state unchanged.
Accepted explicit file intents cancel transient interaction only after context/capability/
intent preflight. A subsequent admission failure is an accepted outcome with a failure
effect, not a false claim that a rejected operation left its draft unchanged. Checkpoints
require idle state and never cancel ink. Unknown, mismatched and replayed continuations do
not cancel work or consume the pending intent. Cancellation is a valid typed reply.

At most one chooser/navigation continuation exists. Queued writes are separate: an ordinary
Save can target the old binding while Save As is queued or its destination choice is pending.
The existing coordinator enforces purpose/capability, FIFO and binding rules. No Rust call
in this tranche opens a dialog, creates a thread, encodes a document or performs filesystem I/O.
Save admission does still synchronously capture the exact immutable snapshot outside input
callbacks, with the previously measured O(document) cost.

## Save ownership and navigation

The session retains up to 1024 admitted tokens until their trusted `SaveCompletion` is
applied, independently of coordinator queue slots. Taking a coordinator completion before
its delivery therefore cannot unblock navigation. If an adapter loses a completion, the
outstanding obligation remains visible and prevents unsafe replacement; there is no timeout
that pretends the write settled. The future runtime must supervise/deliver every completion.
The fixed-capacity token vector bounds this metadata independently of document size.

`CompleteSave(&mut Option<SaveCompletion>)` consumes only after context and issued-token
validation. Rejection retains ownership for delivery to the correct owner. Outcomes include
issued token, actual result/durability/failure and acknowledgement. Checkpoint completions
may cross successful replacement: they release their token but cannot clean/bind the new
session. Explicit write acknowledgements must settle before replacement or close approval.

Navigation waits for explicit writes, then asks about dirty content. Save choices enqueue
real coordinator work; failure cancels navigation while preserving edits. After completion,
dirty state is checked again. Discard decisions apply only to the revision shown; newer
content requires a new decision. A source choice cannot authorize edits committed while its
panel was open. Cancelling navigation does not cancel an already accepted disk write.

Close readiness has a final issued `ConfirmClose` reply. It rechecks revision and outstanding
writes, then returns `CloseConfirmed`. Native code must close synchronously in response before
delivering further editor input. Native app termination still owns its exactly-once platform
reply; no Swift dirty/save decision is required.

## Replacement ownership

`ReplacementTicket` is issued, immutable and non-Clone. It captures session/effect identity,
authorized revision, source, capabilities and settings. A worker prepares it with a fully
validated document (or no document for New), building a fresh editor and binding an ordinary
opened destination before installation. Recovery uses the separate dirty/unbound constructor.
Preparation failure returns an issued failure; the current document remains untouched.
`ReplacementFailed(&mut Option<ReplacementFailure>)` retains that unique terminal event on
stale-context or wrong-owner rejection, then consumes it only after exact pending-token
validation. It can therefore be delivered to the correct owner once, like a save completion.

`Install(&mut PreparedReplacement)` checks issued identity, authorized content revision,
idle interaction and outstanding explicit acknowledgements before taking the candidate.
Failure preserves both candidate and current state. Success preserves current validated
tool/brush settings, swaps a complete session and returns the retired editor. No fallible
bind step follows replacement. Loaded history/selection start fresh and replay is rejected.

The future runtime must reserve bounded retirement capacity before Install, retain a rejected
candidate for cancellation/disposal, and move retired state off-owner for destruction. This
portable API returns ownership; it does not create a retirement thread or claim that callers
already dispose asynchronously. Never install or discard large candidates in pen callbacks.

## Scope and acceptance

Portable tests cover INPUT-001, INK-001/002/003, UNDO-001, SAVE-001/002/003, LOAD-001,
VIEW-001 and ASYNC-001 at the shared route, with existing core invariants retained. Tests
include stale/replayed/cross-session replies, invalid batches, capability rejection, exact
accepted-cancellation policy, old reads/new edits, current-settings preservation, dirty
recovery, late saves, undelivered/lost completion obligations and navigation dirty rechecks.

019B/C still own C layouts/panic boundaries, ordered asynchronous reads/writes, native grants,
worker event dispatch, recovery management and diagnostics sinks. 019D/E own actual native
controls, field/focus behavior, asynchronous sheets and lifecycle E2E. Recovery selection is
an opaque source ID in this tranche; it does not enumerate or discard stored recoveries.
No renderer, native UI, filesystem, physical input or latency acceptance is claimed.
