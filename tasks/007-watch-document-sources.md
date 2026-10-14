# 007: Independent Watch document sources

Status: review. Owner authorized implementation on 2026-10-14; base `03aa825`.

## Outcome

Keep Watch view-only and native. Separate phone, server and cached document access.
Open cached content immediately; prefer iPhone retrieval with a bounded deadline,
then use a configured server for the same selected drawing. No storage service or
production authentication provider exists yet: implement and test the native network
adapter with a deterministic service fixture and explicitly report unconfigured state.
Keep optional iPhone sharing while allowing independent Watch execution.

## Contracts

Rust owns source version decisions through an additive ABI 2 function. Account/library
namespace, drawing ID, immutable revision ID, parent revision, server generation,
kind and SHA256 identify a copy. Timestamps do not order copies. Native adapters own
transport, request correlation, hashing, cache and platform lifecycle. Rust validates
document bytes before atomic publication. Unknown draft lineage is a conflict, never
an automatic downgrade. Legacy copies migrate to isolated phone-only identities.

Phone only exposes explicitly shared snapshots. A queued WatchConnectivity file does
not prove import or block server fallback indefinitely. A stale response cannot switch
the selected drawing/account. Missing network configuration, corruption, cancellation
and unavailable sources preserve the prior authorized drawing. Changed content during
navigation is staged for explicit application. No fake sign-in or live-sync claims.

## Acceptance

Rust version/identity invariants, actual Swift/Rust boundary, cache migration and failure
retention, immediate cache, phone timeout/server success, both offline, phone-only copy,
late reply, selection/account switch, duplicate/stale/conflict, bounded HTTPS and errors.
Build iOS/watchOS simulator and device; native small/large text inspection and independent
launch. Physical paired transfer and real backend authentication remain separate evidence.
Designer and fresh reviewer assess implementation; no merge into main.
