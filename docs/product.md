# Product intent

## Confirmed decisions

- Solo developer, supported by agents, launching on one or two platforms first.
- Infinite vector sketching inspired by Concepts: editable strokes rather than a raster-only canvas.
- Native platform controls wherever possible; custom drawing graphics where necessary.
- Swift/macOS is the first application. Shared behavior is implemented in Rust.
- Windows uses C# + WinUI 3, with a Rust C ABI adapter.
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

iPadOS is the likely second full app. Prototype its input adapter early to challenge
desktop assumptions. Skia-over-Metal is a benchmark baseline; evaluate Vello with the same
geometry and presentation conditions before choosing the production renderer.

## Open decisions

Working name; reference Mac and pen device; oldest supported OSes; monthly agent budget;
distribution/signing; document format; renderer; final keyboard shortcuts and toolbar layout.
Resolve decisions when they affect the current milestone, without blocking unrelated work.

