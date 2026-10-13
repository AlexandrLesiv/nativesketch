# First editable stroke: interaction agreement

Task 001; designed by ui_ux_designer before implementation on 2026-10-14.
This document describes intended behavior; runtime findings belong in the evidence report.

## Native surface

One document window opens with Pen active. An AppKit control row offers Pen/Select,
color, nominal width, and numeric selection scale with Apply. Standard File, Edit,
View, and Tools menus expose applicable commands. Native panels handle Open/Save;
native alerts handle errors and unsaved replacement/close. The canvas, ink, selection
bounds, and four corner handles are custom. No placeholder tools.

The paper is white in both system appearances; controls use system styling. Status
shows the tool, stroke count/selection, and zoom. The empty hint explains Pen, Select,
and Space-drag navigation. Controls must fit at the minimum supported window size.

## Interaction agreement

- Mouse input retains missing pressure and draws at nominal width. Pen pressure p
  uses nominal width × (0.2 + 0.8p), implemented only in Rust.
- One active pointer owns a draft. Pointer-up commits one undo step, including dots.
  Escape, deactivation, and window lifecycle cancellation discard the draft.
- Select hits the topmost stroke; empty click deselects. Pen clears selection.
  Select Next/Previous provide keyboard access to objects.
- Corner handles uniformly resize about the opposite corner, including brush width.
  Numeric scale transforms about the selection center. Preview derives from the
  original geometry; release/Apply commits one step. No-op changes add no history.
  Escape restores the original exactly. Scale cannot collapse or invert geometry.
- Delete is undoable. Undo/Redo apply to completed document operations.
- Space-drag and scroll pan; pinch and View commands zoom. Viewport changes never
  modify document samples. Do not navigate or switch tools halfway through a gesture.
  Window resizing cancels an active interaction before changing its mapping.
- Document coordinates are double-precision, top-left/y-down points. Backing pixel
  scale is presentation-only. Canvas resizing must not shift stored geometry.
- File commands cancel temporary interactions. Invalid Open preserves the current
  document. Save uses a same-directory temporary file and atomic replacement.
  Failure preserves the last good file and dirty state. Replacement/close/quit uses
  Save / Cancel / Don't Save, with cancelled or failed Save cancelling replacement.

## Keyboard and accessibility

Cmd-Z, Shift-Cmd-Z, Cmd-S, Shift-Cmd-S, Cmd-O, and Cmd-N use standard menus.
Pen/Select letters and Space/Delete belong to canvas focus, never text fields.
View commands and object selection provide keyboard alternatives. Controls expose
labels and enabled states; numeric fields preserve normal text editing. Reject invalid
numbers without changing the last valid value. Canvas exposes its stroke/selection/zoom
state. Selection uses visible bounds and handles, not color alone.

## Acceptance checklist

1. Mouse dot and stroke remain visible, selectable, and editable.
2. Replay missing and varying pressure; validate samples and rendered width.
3. Cancel drawing/resize by Escape and deactivation; history/document stay unchanged.
4. Handle resize and numeric scale both undo/redo exactly.
5. Pan, zoom, and window resize preserve document coordinates and mapped hit tests.
6. Save/reopen retains geometry/style; corrupt Open preserves current content;
   unsuccessful Save preserves previous bytes and dirty state.
7. Menus, object selection, numeric scale, and text editing work by keyboard.
8. Inspect empty/selected/error states, narrow/large windows, and light/dark appearance.

## Explicit prototype limits

CoreGraphics is a disposable painter of Rust geometry for this milestone. It is not
selected as the production renderer. Task 002 remains the required comparison of
Rust-Skia/Metal and pinned Vello before any permanent backend commitment. No GPU
winner, frame budget, physical pen feel, or pen-to-screen latency is inferred here.
