# Linux portable adapter migration — task 022

This slice replaces GTK's legacy `Editor` mutations and host-owned save guards with
`sketch-app::PortableApp`. It preserves the existing GTK/libadwaita controls, retained
color control, canvas keyboard focus scope, 0.05–32 zoom range and Cairo appearance.
It does not migrate Linux rendering to Skia or claim Linux native acceptance.

## Ownership and boundaries

- `activation` retains one application owner across activation and shutdown drain.
  Repeated activation presents the existing editor. Activation during drain cannot
  construct a second owner that could write concurrently with the old one.
- `command_owner` owns `PortableApp`, its session and request correlation. Native
  buttons and `shortcut_intent` use the same typed `EditorAction` dispatcher. Pointer
  translation uses typed edit/input commands. Availability comes from the projection.
- `model` owns only native gesture/viewport state alongside that command owner.
  Native widget updates happen after releasing model borrows.
- `file_effects` and `file_dialogs` present issued `HostEffect` values, preserving the
  original token/session/request for replies. They do not implement dirty-navigation
  policy. Save retains its previous chooser behavior by dispatching `SaveAs`.
- `provider` retains exact resource ID/generation leases. IDs are process-unique;
  paths remain private host data. Unknown aliases are ordered by the one facade.
  No canonicalization or filesystem access occurs in GTK/input callbacks.
- One bounded worker receives issued work only after an empty retained host slot is
  available. Read, encode, prepare, atomic write and retirement run off-owner.
  Reads are limited to maximum document bytes plus one. Writes use
  `sketch-services::storage::{DestinationAccess, write, NoFaults}`.
- Completed work remains uniquely owned until `complete_work` consumes it. Rejected
  results and retryable apply work remain retained. Resources are removed only for
  matching issued `take_release` records. Failed save receipts are surfaced explicitly;
  they cannot become successful persistence acknowledgements.
- Close hides the window only after shared confirmation. A GLib application hold and
  bounded polling remain until actual work/retirement/resource drain and worker exit.
  Shutdown never joins an active worker or treats dropping work as termination.
  A 30-second stall emits at most four warnings; the live hold remains rather than
  falsely reporting completion. Ordinary operation/failure diagnostics are redacted
  and capped at 128 process records; no sample/path/document logging is emitted.
- `scene_painter` traverses immutable `RenderScene` in original paint order, suppresses
  the original transform-preview stroke, and fills live actual/predicted geometry
  once. Cairo retains f64 coordinates; geometry origin must not be added twice.
  Selection handles remain native adapter presentation. Scene cache limits remain
  the shared bounded defaults.

## Verification and traceability

All tests below are pure adapter/provider/Cairo tests. They do not initialize GTK,
launch a window, exercise accessibility, or prove native focus delivery.

| Test | Acceptance/invariants |
| --- | --- |
| `adapter_tests::adapter_cancel_transform_roundtrip` | Original eight assertions preserved or strengthened: sample timing, count/cancel, transform undo, malformed load preservation, viewport anchor, failed save/dirty; INK-001, UNDO-001, LOAD-001, CANCEL-001 |
| `adapter_tests::typed_button_and_shortcut_intents_cancel_drafts_and_undo_once` | Real shortcut translation versus button typed intent; active draft cancellation/history, zoom limits; INK-001, UNDO-001 |
| `provider_tests::delayed_save_keeps_newer_edits_dirty_and_release_is_once` | Save A completing after edit B keeps B dirty; SAVE-001 |
| `provider_tests::stale_chooser_cannot_consume_current_navigation` | Old chooser token cannot resolve newer continuation; ASYNC-001 |
| `provider_tests::cancelled_active_read_keeps_current_document_and_retires_candidate` | Actual read completes after cancel, candidate rejected/retired; LOAD-001, ASYNC-001 |
| `ordering_tests::unknown_path_alias_exports_cannot_start_out_of_order` | Same-path separate leases cannot overwrite newer snapshot out of order; SAVE-003 |
| `ordering_tests::close_waits_for_actual_write_then_retires_every_resource` | Cancel close while write stays outstanding, finish actual write, close/retire/release/join; FFI-001 |
| `load_order_tests::older_read_finishing_after_newer_load_cannot_replace_new_document` | Newer load finishes first, late older result cannot replace it; ASYNC-001, LOAD-001 |
| `scene_tests::cairo_scene_coordinates_and_transform_preview_suppress_original` | Actual Cairo pixel coordinates, preview suppression, cancel restores pixels; RENDER-001, CACHE-001 |
| `activation::tests::activation_never_creates_an_owner_while_old_work_drains` | One owner across repeat activation and closing interval; SAVE-003, FFI-001 |

Commands: `cargo fmt --manifest-path apps/linux/Cargo.toml -- --check`,
`cargo clippy --locked --manifest-path apps/linux/Cargo.toml --all-targets -- -D warnings`,
`cargo test --locked --manifest-path apps/linux/Cargo.toml`, and
`cargo build --locked --manifest-path apps/linux/Cargo.toml`.
All handwritten Rust files remain below 200 physical lines after rustfmt.

Evidence lives in `artifacts/evidence/022-linux-portable/`: final build/test/Clippy
logs and identity JSON identify the exact committed source, host, toolchain and
binary hashes. Earlier development attempts caught an incorrect test byte offset
(80 versus the expected 0.027 timestamp), missing presentation of failed save
receipts, and compiler/Clippy findings. Those were repaired; early tool outputs
remain in the agent execution history, not falsely represented as artifact logs.

## Remaining host gaps

Compilation and ten pure tests run on macOS arm64 with GTK 4.24.1/libadwaita 1.10.0.
This is not Linux Wayland/X11 execution. Native numeric-field shortcut priority,
portal choosers, activation/focus, pen/touch/backlog, GTK close behavior and desktop
accessibility need an actual Linux host E2E session. No performance claim is made.
Physical pen latency, Linux Skia migration and cross-process file coordination are
outside this slice. Atomic access deliberately rejects unsupported/symlink destinations
and reports failure; it does not silently fall back to non-atomic writes.
