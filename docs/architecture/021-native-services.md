# Task 021B: native storage and recovery services

`sketch-services` depends only on `std` and `sketch-core`. The core remains portable and
owns document semantics; the services crate owns local filesystem effects and an OS thread.
It adds no ABI or Wasm messages and does not adopt the services in a native UI. Task 019
provides the typed host bridge and recovery presentation.

## Owner and effect boundary

The editor owner routes native controls, shortcuts and future agent intents through the
same typed command boundary. Save and Save As differ by semantic purpose, not input source.
The native destination chooser produces an opaque destination identity and a retained
access grant; the process-wide owner maps that identity to `DestinationAccess`.

1. Reserve and capture through `SaveCoordinator::request` on the editor owner. Exact
   immutable snapshot capture is still O(document), outside pen callbacks. Native explicit
   file actions cancel transient input first; checkpoint scheduling defers active input.
2. `start_next` transfers one unique `WriteJob`. Pair its token's destination with the
   retained access grant and construct `Submission { job, target }`. The host must validate
   this association: paths deliberately do not enter portable tokens. Canonical aliases
   must share an identity/coordinator. The service cannot infer a path from an opaque ID.
3. `SerialStorageWorker::try_submit` never waits. Failure returns unique job ownership intact
   for retry or explicit failure completion; never drop an active coordinator reservation.
4. `poll_receipt` returns `Ok(None)` while pending, `Ok(Some(receipt))` when complete, and
   `Err(WorkerDisconnected)` after disconnection. Return the owned receipt to `finish`, then
   consume the issued completion on its editor owner. Retain grants through terminal receipt.
5. Worker encoding, validation, checksumming, filesystem writes and flushing never borrow
   the live editor. The worker rejects document/checkpoint purpose mismatch and checkpoint
   metadata whose revision differs from the issued job. It supervises operation panics and
   returns `Uncertain(WorkerStopped)` rather than pretending the old file is unchanged.

There is one bounded input slot and one bounded receipt slot. Output backpressure blocks
only the storage thread. The coordinator normally admits one active job; service slot
capacity is not permission to run independent coordinators against the same destinations.
`request_shutdown` rejects further submission, disconnects input and lets accepted work
finish. Continue polling terminal receipts until stopped/disconnected and account for every
issued job. `Drop` never joins, never rolls back an active write, and is not a durability
promise: dropping before draining may abandon queued work or its acknowledgement. Receipt
polling must continue during native deferred termination. No synchronous wait belongs on UI.

## Staged writes and filesystem assumptions

`DestinationAccess` validates absolute local paths, every existing parent, and the target.
Observed symlinks, unsafe path components and nonregular entries are rejected. Writes create
an exclusive temporary sibling, write all bytes, sync it, replace the destination, then sync
its directory. Pre-replacement failure preserves the old target; cleanup failure is reported.
Post-replacement/directory-sync failure reports written content with unconfirmed durability.
`StoreReport` preserves failure category and stage without logging paths or raw error text.

On Unix, new stages start at mode 0600 and replacement preserves existing standard permission
bits. This does not preserve ACLs, extended attributes, provider coordination or security
scope automatically. Sandboxed Apple providers and Windows metadata need native adapters and
host tests before claiming those guarantees. Rename and directory-sync capabilities vary.

These std-only checks assume a trusted local directory and one exclusive service owner.
They are not descriptor-anchored/no-follow operations and do not prevent a hostile process
from swapping a path between validation and use. Do not advertise adversarial filesystem
safety. Unexpected symlinks are tested; concurrent hostile mutation is not supported.
Ordinary-document process crashes may leave private sibling staging files. Recovery storage
has bounded staging management; this is not automatic cleanup of every user directory.

## Recovery ownership, bounds and lifecycle

The host supplies a cryptographically random nonzero 128-bit `RecoveryId`, independent of
runtime session counters. Exclusive allocation rejects collisions. The Rust type validates
shape, not entropy. `RecoveryStore` is a cloneable descriptor of a private root and limits;
all initialization, allocation, publication, reading, retention and management methods are
synchronous I/O and must execute on an exclusive worker owner. Cloning grants no concurrency
or cross-process writer guarantee. Keep its native directory access grant alive for all work.

Versioned envelopes contain opaque identity, generation, revision, local display metadata,
validated document bytes and a checksum. Names are bounded and never used in paths/logs.
Checksums detect corruption, not malicious alteration. Candidates validate both envelope and
existing document codec, falling back to older valid content. Restore uses
`EditorSession::new_recovered`: edit capability required, fresh session, dirty and unbound.
The native open flow must also validate issued read/session identity before replacing state.
There is no asynchronous read-job API in this phase; the typed bridge must schedule reads
on its shared bounded service rather than decode documents on the UI owner.

Defaults retain two valid generations per session, at most 20 sessions and 256 MiB including
staging/metadata. Admission conservatively reserves bytes before publication; existing data
is preserved at capacity. Sessions are scanned to 64 entries and root listing to 20 entries.
Corrupt or overfull sessions do not block loading unrelated candidates. Listing reports
rejected sessions/generations/markers; strict new-write admission can still reject the store.
Unexpected external files beyond scan bounds are not silently scanned without limit.

Resolution metadata is versioned and identity/checksum validated. It hides only the exact
resolved candidate. Expiry after seven days requires the exact latest generation to match;
corrupt metadata cannot authorize deleting newer unresolved content. Explicit confirmed
management uses `discard_step` (at most 32 recognized regular entries), repeated by the host
until complete. `cleanup_staging_step` is bounded and requires no active write in that store.
Never expire the last unresolved generation to make room.

`CheckpointSchedule` is a pure injected-clock state machine: 2-second debounce or a 30-second
scheduling target, one active checkpoint and one coalesced newest pending revision. Active
interaction and pending explicit saves defer admission. Failure suspends automatic retries;
explicit resume is required. Replaced-session and stale completions are rejected. These
intervals do not promise maximum data loss or physical durability.

## Diagnostics and memory

Worker receipts carry at most two fixed-field events: opaque correlation IDs, revision,
operation code, duration, count, failure category, stage, durability and cleanup status.
No paths, names, content, coordinates or per-sample records enter this schema.
`DiagnosticRing` bounds events and per-code rate, tracks drops and sink failures, and retains
an event after sink error or panic. Drain it off the ink path. `RotatingSink` bounds owned
files/bytes and rejects oversized pre-existing files; it does not delete unrelated files.
Task 019 subsequently wired the bounded sink and exposed failure counters in the macOS
runtime. Other hosts still require their own adoption and verification. No hosted telemetry
or global logging settings are installed.

Coordinator snapshot limits exclude allocator overhead and worker temporaries. Encoding,
envelope construction and validation may hold additional bounded document-sized buffers
and decoded geometry; this is not a process RSS budget or a latency claim. Task 020 may
optimize copies after equivalent ownership and validation tests.

## Acceptance and remaining work

Services tests cover SAVE-001/002/003, LOAD-001 and ASYNC-001 with injected disk/access,
cleanup and durability failures, panic supervision, cancellation/ordering contracts from
021A, bounded diagnostics, corrupt/oversized recovery, capacity, retention and restoration.
Actual disposable child-process kills exercise ordinary and checkpoint publication. These
are process-crash tests, not power-loss or real native UI tests. Task 019 subsequently
implemented native save/reopen/recovery presentation, deferred quit, shared command routing
and keyboard E2E. See [its separate native evidence](../evidence/019-native-desktop-validation.md).
These service tests do not imply physical-input, sandbox-provider, Windows or other-device
acceptance.
