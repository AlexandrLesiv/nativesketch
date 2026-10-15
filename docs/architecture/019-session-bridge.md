# macOS typed-session integration contract

Proposed implementation boundary after reviewed task021 services. This document does
not claim new symbols or migrated native callers. The existing ABI2 and Wasm1 stay intact.
The initial native extraction remains a compatibility step; it must not create another
Swift authority for brush, tool, document, history or saved revision.

## Ownership and interface direction

- An additive, independently versioned C session interface wraps `EditorSession`; legacy
  `SketchEditor` symbols and numeric statuses retain their existing semantics.
- One Swift `@MainActor` bridge owns each session handle and frees it once. All session
  calls, including destruction, occur on that owner. Worker code never receives the handle.
- One application-owned service bridge owns the process-wide save coordinator and bounded
  storage worker. Windows submit owned snapshots and receive correlated results. A window
  closing does not permit an active writer to be forgotten or a newer writer to overtake it.
- New headers are cohesive (session values/commands, owned query results, services), each
  at most200 physical lines. Keep stable integer fields and explicit lengths; no Swift,
  AppKit, Rust enums, `usize` identities or borrowed asynchronous pointers cross the ABI.
- Every fallible entry contains panics and returns a defined status. A caught mutation
  panic poisons the relevant handle; no further mutation is allowed. Resource destruction
  remains supported. Pointer validity and unique ownership are caller preconditions.

## Commands and projections

Use typed wrapper methods over named C functions or a validated tagged command record;
never deserialize arbitrary Swift objects into domain commands. Raw enum tags and numeric
values are validated before mutation. Context contains session, request and interaction
identities; stale input must fail even if a pointer ID or document revision repeats.

A small projection supplies Rust tool/brush, capabilities, dirty/revision/session,
interaction and action availability. This projection is O(1); selected geometry, scene
and committed snapshots remain explicit potentially expensive queries. A presenter updates
native controls from the projection. Incomplete field text belongs to the native field;
validated commits send `SetBrush`. The canvas input adapter reads the shared projection,
never inspector widgets. Native text undo, selection, deletion and shortcuts retain first
responder precedence. Explicit native actions may cancel an active transaction before
requesting a state change; the bridge must not silently cancel on every command.
Unrelated projection refreshes must not overwrite an active field editor's incomplete text.
Before a gesture moves focus to canvas, native end-editing validation must succeed; rejected
text retains field focus and must not start ink with a silently different brush setting.

## Save and completion ownership

The native file panel resolves a private access grant and canonical destination identity.
The application service retains that grant until its terminal receipt is drained, including
after a window closes. Existing-file aliases and nonexistent-file identity use one documented
native resolver: resolve the existing parent identity plus the proposed leaf for new files;
reconcile identity after replacement. A globally serial worker remains the final ordering
guarantee even when two grants were assigned different identities for one destination.
Snapshot admission runs on the session owner, outside pen callbacks, with capacity reserved
before copying. It captures the exact requested revision. The service worker owns encoding
and staged writes; Swift never serializes a document or marks it clean independently.

An issued completion is consumed once. Its session, revision, destination, binding generation
and sequence come from the coordinator, not caller-supplied claims of a successful write.
Native polling is nonblocking and bounded. Route results by session identity, acknowledge
on the mutation owner, then refresh the projection. A stale or closed window does not show
an obsolete failure panel. A completion can still represent an actual disk write after
cancellation; retain that outcome rather than assuming cancellation undid it.
Use one app-owned callback source while jobs are outstanding, with a fixed maximum receipt
count and time budget per turn; reschedule remaining receipts on a subsequent main-loop turn.
Do not poll per input sample or leave a continuous polling timer when storage is idle.

Save As adopts the new destination only after confirmed success and Rust acknowledgement.
Ordinary Save continues to use the old binding while Save As is pending. Do not dispatch
another Save As for that session until its completion is consumed. No native URL change
may run ahead of this domain result. Error sheets offer a deliberate retry/new destination
without dropping edits or reusing an already-consumed completion.
The window title continues to identify the old destination until confirmed adoption.

## Open, recovery and navigation

A bounded reader produces a fully validated owned candidate off the UI thread. Its request
captures the previous session plus a native navigation generation. On the owner, reject
obsolete candidates before replacement; do not bind a delayed read to the current document.
Use one bridge operation for `ReplaceDocument` plus `bind_loaded_destination` after the
candidate and destination are valid, with no native input between those steps. Keep the
previous document on validation/read failure. Test the binding preconditions and ensure
there is no fallible post-replacement step that can leave a partially applied load.

Recovery uses the separate recovered-copy constructor: editable, dirty, no destination,
new session identity. Never call normal load binding for a recovered copy. Explicit discard
and candidate retention remain service operations; opening a recovered copy does not imply
that its only recovery generation may be deleted immediately.

Close/new/open/quit use a native navigation state machine and asynchronous sheets. Pending
writes settle without UI waits; cancellation can cancel navigation while retaining the open
document. Recheck dirty state after completion because newer edits can exist. Deferred app
termination must finish or deliberately cancel this workflow rather than blocking the main
thread or freeing a live session behind callbacks.
Track one navigation intent at a time. Every `.terminateLater` response receives exactly one
`reply(toApplicationShouldTerminate:)`, including cancellation and service failure paths.

## Acceptance before replacing legacy consumers

- C/Rust layout and failure/capacity/ownership tests; Swift bridge tests; ABI2/Wasm1 fixtures.
- Stale session/interaction/read/completion rejection and teardown with pending work.
- Brush controls project Rust state; drawing captures committed settings, predictions stay
  temporary, cancel preserves history and a stroke is one undo operation.
- Real native Save/Save As shortcuts and text-field shortcuts, with visible canvas and file
  assertions; core/bridge tests alone do not certify native event delivery.
- Actual save failures, delayed completion during edits/navigation, recovered dirty/unbound
  copies, and bounded queue/log behavior. Keep physical pen latency pending a camera.

Contract refinement must be agreed by Rust and macOS owners before parallel implementation.
This phase does not change the temporary Core Graphics painter; production Skia follows020.
