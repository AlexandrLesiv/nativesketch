# Portable save policy and document bounds

Task 022 typed contract amendment. Existing native-v1/macOS, legacy ABI 2 and Wasm 1
remain unchanged. This is Rust behavior/query evidence, not native UI acceptance.

## Immutable per-session save policy

`SaveDestinationPolicy::ReuseBinding` is the default. Existing
`ApplicationSession::new` and `PortableApp::create_session` retain that behavior.
`ApplicationSession::new_with_save_policy(..., policy)` and
`PortableApp::create_session_with_save_policy(config, policy)` opt into
`AlwaysChoose` without changing SessionConfig or mutable document data.
The policy survives New/import because it belongs to ApplicationSession.

| Intent | ReuseBinding | AlwaysChoose |
| --- | --- | --- |
| Save without binding | ChooseDestination(SaveAs) | ChooseDestination(SaveAs) |
| Save with binding | Queue Save to binding | ChooseDestination(SaveAs) |
| Unsaved-choice Save | Same rule as direct Save | Same rule as direct Save |
| Bound Save during another dialog | Existing exception remains | Busy; preserve draft/dialog |
| SaveAs / ExportCopy / Checkpoint | Existing semantics | Existing semantics |

One helper chooses the reusable destination for both direct and guarded Save. Hosts
translate the same typed intent; they do not fake SaveAs acknowledgments or reproduce
navigation logic. This restores the chooser behavior required by the existing Linux
and Android adapters; host adoption is separate.

An accepted destination choice snapshots/queues an actual SaveAs operation. Only a
matching confirmed write may bind the new destination and acknowledge its revision.
Failed, uncertain or cancelled writes preserve the prior binding and dirty state.
An older successful snapshot cannot authorize navigation past newer edits. Cancellation
and stale chooser replies retain existing generation validation. Rejected overlapping
Save under AlwaysChoose validates before cancellation and leaves active ink intact.
Completed unused provider leases release exactly once; the live binding retains its lease.

## Explicit full-document bounds query

`EditorQuery::DocumentBounds` returns `QueryResult::DocumentBounds(Option<Bounds>)`.
`PortableApp::document_bounds(session)` exposes the same typed query and rejects foreign
or detached sessions. Empty documents return None, not a fabricated zero rectangle.
Editable and read-only sessions may query.

The rectangle is the union of committed stroke outlines, including captured brush width,
pressure and committed transforms. It uses the same core geometry bounds calculation
as rendering, preserving f64 coordinates. Selection, viewport culling, temporary actual
samples, predictions and transform previews do not affect the result. Querying changes
neither document/history nor interaction state and requires no scene/cache construction.

This is an explicit O(total committed samples) traversal on the mutation owner. It does
not run on input append, projection or every frame. The host chooses when an existing
Fit action needs it and keeps platform padding/zoom behavior. No background-query or
revision cache is introduced; a later asynchronous transport must carry document identity
and revision so delayed responses cannot resize a replacement document.

## Acceptance and limitations

`quality/features/portable-policy-bounds.json` maps the exact core/facade tests to
SAVE-001/002/003, ASYNC-001, INK-001/002, VIEW-001, RENDER-001 and CACHE-001.
Coverage includes previous binding plus direct/guarded save, native-dialog overlap,
chooser cancellation/staleness, failed/uncertain/cancelled/older writes, exact lease release,
large fractional coordinates, pressure/transforms, undo/redo/delete/replacement, viewport
exclusion, live drafts/predictions and read-only source replacement.

Wire opcodes, C/Wasm consumer coverage, Linux/Android adoption and native E2E remain separate.
Apple's staged temporary export followed by a picker and confirmed nonbinding save is a
different contract and remains pending. This policy does not claim that behavior.
