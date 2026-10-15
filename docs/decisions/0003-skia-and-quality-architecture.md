# 0003: Skia and shared editor quality architecture

Status: accepted direction by the owner in conversation; implementation pending.
Date: 2026-10-15. Implementation base: `codex/github-pages` at `712059d5`.

## Decision

Use Skia for the editing renderer, including standard CanvasKit on web. Preserve native
AppKit/UIKit/Material Views/WinUI/GTK/HTML controls. Do not import the renderer-results
branch into the new implementation line. Historical comparisons remain on their branch;
the choice does not assert universal performance or physical pen latency superiority.

Share semantic tools/actions, validated settings, availability, document/editing behavior
and command/query boundaries in Rust. Keep native layout, localization, focus and event
capture per platform. Use composition, presenters, adapters and explicit state machines.
Agent transport is deferred. Watch/TV remain view-only; Watch independently loads validated
cache with phone-first/same-identity network fallback, production auth separately scoped.

Enforce <=200 physical lines per handwritten source file including tests/scripts; explicit
generated/vendor/lock/data exclusions. Migrate by responsibility, preserving current ABI,
Wasm protocol and bytes before intentional behavioral or performance changes.
Use Node.js for owned automation and Oxc/Vite for web/tooling; retain specialized Python
analysis only where already needed until verified replacement. Keep native bridge/toolchains.

Feature acceptance IDs, executable mappings and revision-specific results form the coverage
record. Require applicable native E2E, invariants, recovery, diagnostics, accessibility and
measured performance evidence; do not equate replay with UI or GPU timing with photons.
Use existing agents: lead/Rust plan architecture, reliability owns infrastructure, each
implementer owns tests/logging, designer reviews UI and reviewer independently assesses.

## Consequences

This supersedes renderer OPEN language in active scope documents, not historical evidence.
Production integration, minimum OS/device matrix, reference budgets, deployment and network
service are not completed by this decision. Viewers may consume validated previews where
appropriate; Skia selection does not require a continuous GPU renderer on Watch.
No main merge, release publication or unattended service is authorized by this document.
See [architecture](../architecture.md), [plan](../refactoring-plan.md) and [quality](../quality.md).
