# Product intent

## Confirmed decisions

- Solo developer, supported by agents, launching on one or two platforms first.
- Infinite vector sketching inspired by Concepts: editable strokes rather than a raster-only canvas.
- Native platform controls wherever possible; custom drawing graphics where necessary.
- Swift/macOS is the first application. Shared behavior is implemented in Rust.
- First-stroke hosts use UIKit on iOS/iPadOS, Material Views on Android,
  C#/WinUI 3 on Windows, GTK4/libadwaita on Linux, and HTML + Rust/Wasm in browsers.
- These parallel prototypes were authorized on 2026-10-14; shipping every platform
  simultaneously remains outside this milestone.
- Responsiveness and consistent drawing behavior are primary quality requirements.
- Start with Codex. Agents implement/test autonomously; the owner approves merges.
- GPT-6 Astra is the requested model. Agent budget and reference hardware are undecided.

## First useful product

Pressure-sensitive pen, fixed-width pen, pan/zoom, layers, selection/transforms,
whole-stroke erase, undo/redo, reliable save/open/autosave, image import, PNG/basic SVG export.
Build these as small end-to-end milestones rather than all at once.

## Deferred

Collaboration, cloud sync, brush marketplace, advanced text, PDF import, shape recognition,
advanced vector booleans, textured brush engine, and simultaneous launch on every platform.

## Proposed, not yet committed

iPadOS/iOS is the intended second full app. Its touch prototype challenges desktop
assumptions; production platform acceptance remains separate from simulator checks. Skia-over-Metal is a benchmark baseline; evaluate Vello with the same
geometry and presentation conditions before choosing the production renderer.

## Open decisions

Working name; reference Mac and pen device; oldest supported OSes; monthly agent budget;
distribution/signing; document format; renderer; final keyboard shortcuts and toolbar layout.
Resolve decisions when they affect the current milestone, without blocking unrelated work.

