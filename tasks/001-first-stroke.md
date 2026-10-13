# 001: First editable stroke on macOS

Status: ready. This task is not implemented by the bootstrap.
Roles: ui_ux_designer, rust_engine, macos_ui, reviewer.

## Outcome

Draw a pressure-sensitive stroke, select and resize it, undo the change, save the document,
and reopen it with geometry and style preserved. Mouse drawing must also work.

## Start with design and contracts

Designer specifies native control mapping, drawing/navigation/selection transitions,
keyboard access, cancellation, empty/error states, and observable acceptance cases.
Rust and macOS owners agree the event/coordinate/ownership contract before parallel edits.
Break the feature into smaller commits; integrate the smallest path first.
Use task 002's renderer decision before a permanent backend commitment.

## Acceptance cases

- Completed stroke remains editable; cancelled input does not create a committed stroke.
- Pressure-capable devices vary width; missing pressure uses a documented fixed behavior.
- Pan/zoom and window resizing do not shift the stored geometry.
- One completed stroke and one transform each produce coherent undo/redo steps.
- Save/open retains geometry and style; unsuccessful save preserves the last good document.
- Native UI can be operated with keyboard where applicable; no inert placeholder tools.

## Evidence

Relevant core invariants, input replay fixtures, Swift/Rust integration, native app inspection,
save/reopen and failure tests. Capture provisional timing on named hardware; identify missing
physical-tablet verification honestly. Fresh reviewer plus owner drawing/merge acceptance.

## Deferred

Textured brushes, collaboration, cloud, advanced text, PDF, and full platform ports.

