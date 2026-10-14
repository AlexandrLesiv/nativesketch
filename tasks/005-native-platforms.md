# 005: Native first-stroke hosts

Status: active. Authorized by owner on 2026-10-14, based on first-stroke branch 59246c3.
This extends one editable-stroke milestone across hosts; it does not publish releases
or accept the production renderer before task 002.

## Outcome

Shared Rust drawing/document/history/serialization with native macOS AppKit, iOS/iPadOS
UIKit, Android Kotlin Views/Material, Windows C#/WinUI 3, Linux Rust GTK4/libadwaita,
and browser TypeScript/semantic HTML + Rust/Wasm hosts.

## Acceptance

- Draw/dot/pressure/cancel, select/uniform scale, undo/redo/delete, pan/zoom.
- Document bytes and geometry are portable across hosts; failure preserves prior state.
- Native touch adaptation; second contact cancels draft before navigation; resize/rotation
  changes only viewport, cancelling temporary input before conversion changes.
- Applicable keyboard/accessibility controls; dirty replacement guards; no inert controls.
- Apple standard native chrome adopts current Liquid Glass styling; Android uses Material
  Views. Do not simulate native widgets with canvas or duplicate document logic in hosts.
- Reproducible builds/tests and explicit per-platform distinction between built, executed,
  inspected, and unverified. Lack of another OS host must never become a false runtime claim.
- Designer audit and fresh independent review; human merge approval, main unchanged.

## Rendering boundary

Use disposable platform path painters consuming Rust geometry. Native UI framework choices
are agreed; production canvas backend selection remains task 002's measured decision.
