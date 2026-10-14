# 0003: Initial renderer comparison proposal

Status: proposal; owner acceptance pending. Production renderer selection OPEN.
Date: 2026-10-14.

## Evidence

[Task 002 comparison](../evidence/002-renderer-comparison.md) measures Rust-Skia
0.153.3 Ganesh/Metal and Vello GPU 0.3.0 Sparse Strips on the available M2 Pro Mac.
The original matrix covers 320 paired primary processes and six sustained processes.
The report distinguishes original measurements from guarded replays and retains
presentation-state anomalies, visual differences and incomplete/unsupported evidence.
It is the authority for exact revisions, numerical results and verification status.

## Proposed direction

Update: the next planned decision experiment is [Skia versus Core Graphics physical
pen latency](../benchmarks/skia-coregraphics-pen-latency-plan.md) on the connected Wacom
Cintiq with Pro Pen 3. Vello is outside that experiment; its historical evidence remains.
The earlier Skia preference below concerns the tested Skia/Vello adapters, and does not
establish a Skia advantage over Core Graphics. No physical latency results exist yet.

### Earlier GPU comparison recommendation

Use Skia as the baseline for the next scoped interaction integration experiment, while
retaining Vello GPU as a candidate. Skia's current adapter has lower pan/zoom and live-update
submission costs; Vello has lower dense retained-scene costs. Dense first-use/memory costs
and high-resolution transparency have different tradeoffs. This does not select a universal
winner or authorize replacing the current production prototype.

Before a permanent choice, compare the application's intended incremental rendering policy:
retained document scene, transient live-ink overlay, view-transform caching and drawable
acquisition after CPU scene preparation. Reuse recorded inputs and equivalent quality;
measure tails, first use, memory and native presentation on an uninterrupted host.
The follow-up is a scoped task for owner approval, not a permanent multi-backend abstraction.

## Consequences and acceptance boundary

The CoreGraphics prototype and document/FFI contracts remain unchanged. Renderer objects do
not enter the editable document. Neither this proposal nor macOS measurements certify other
platform APIs, package size, energy consumption, physical pen latency or production lifecycle
handling. Skia's native binary/build dependency and Vello's preprocessing/integration model
must be assessed on each target platform.

The owner accepts or revises this proposal separately from approving the experiment code.
Acceptance of this document would authorize a follow-up plan only; production replacement,
merging into main and release publication still need their own explicit scope/approval.
