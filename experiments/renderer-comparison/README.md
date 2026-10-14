# Native renderer comparison experiment

This standalone package does not select or modify the product renderer. The exact execution
matrix and acceptance gates are in `tasks/002-renderer-comparison.md`. Build through
`./scripts/build-renderer-comparison`; the AppKit host exposes the same parameters as the CLI.
No GPU work belongs in the CPU-only Rust unit tests.

The baseline is Rust-Skia 0.153.3 Ganesh/Metal. The candidate is Vello GPU 0.3.0 Sparse Strips
with CPU path preprocessing and wgpu 30.0.0 Metal rasterization/compositing. This deliberately
uses the maintained GPU family, not the older compute-centric research renderer or Vello CPU.
Cargo.lock pins all resolved dependencies; it includes wgpu internals at 30.0.1.

The official minimal `ganesh-metal` Skia binary returned 404. This experiment enables JPEG
and PDF APIs solely to match the published Ganesh/Metal binary package; neither API is used
by the workloads. The bindings name upstream `m153-0.101.2`. The official package is
`skia-binaries-b7f043e0b1e2a850e702-aarch64-apple-darwin-ganesh-jpegd-jpege-metal-pdf.tar.gz`,
release digest `sha256:59ac182e5b81445ba8bb84c5a65468e3aec7b6ee1b642868603a3725f29a1059`.
This packaging dependency and the failed minimal-binary/source-build attempt (missing ninja)
are integration costs, not renderer performance measurements.

Both adapters use one wgpu-created Metal device and command queue. The same tracked BGRA8
texture is rendered, copied by the same Metal blit presenter, and presented through the same
CAMetalLayer settings. The texture is initialized once through wgpu before external Skia
access so wgpu's initialization tracker cannot clear Skia content during later readback.
The retained raw Metal objects are never destroyed through HAL handles. All operations and
host FFI calls are serial; command buffers are released after retirement. Metal resource
hazard tracking is checked at runtime and protects output reuse across in-flight frames.

Each complete Rust stroke outline is one nonzero fill. Static vector paths/resources are
retained and rasterized every frame. Pan/zoom keeps Skia paths and changes its canvas matrix;
Vello keeps the source paths but reprocesses pixel-space sparse strips after the transform
changes. That necessary backend work is included in scene timing. Images start as identical
straight-alpha RGBA bytes; Vello's explicit pixel metadata requests premultiplication for its
atlas. The synthetic PNG includes translucent regions, color gradients and a checkerboard.

The common GPU **timeline span** starts with a tiny Metal marker before all backend scene
preparation and ends after the common blit. It includes CPU preparation/submission gaps and
queue scheduling; it is not pure GPU execution time. CPU stages, wall frame starts, GPU
completion and positive Metal presentation callbacks are reported separately. No physical
pen latency is measured. Completion mode awaits each submitted frame; paced mode permits at
most two in flight with display synchronization. Missing callbacks/timestamps stay empty
with reasons. The bounded final callback drain is excluded from measured elapsed time;
final GPU retirement is recorded separately and included in elapsed time.

For instrumentation calibration only, `RENDERER_GPU_TIMING=off` skips the begin-marker
command buffer and timestamp harvesting. It retains the same renderer work, common final
blit/presentation, and completion/retirement policy. All GPU timing columns are empty with
`calibration-disabled` as their reason; the manifest records the setting and provenance.
The default (or explicit `on`) enables profiling. Any other value is an initialization error.
Pair on/off trials of the same workload and settings to quantify the marker's CPU/wall pacing
cost; do not combine calibration-off samples with the primary profiled matrix or describe
them as GPU execution measurements. The separate external RSS sampler must also be calibrated.

The 240 live-stroke expected geometry/state hashes are computed through the actual core
input/outline path before GPU initialization, then the logical fixture is reset. Timed live
frames look up that validated identity while still executing real input/outline/backend work.
The final actual geometry hash is recomputed outside timing and must match. CPU tests compare
every frame in the cycle and phase resets. Precomputation time is recorded separately; this
is expected-trace identification, not a claim of actual per-frame runtime hashing. Other
fixtures retain their static geometry hash and hash only small changing transforms. Frame
rows and callback atomics remain until output; their count/capacity is recorded so sustained
RSS is not mistaken for pure renderer storage. OS peak RSS also includes common fixture
state, the host, instrumentation and untimed final readback/PNG output. External sampler
measurements are needed to identify the timed rendering interval's memory separately.

`final.png` and `readback.json` are captured after timed work. The built-in nonblank/opaque
background validation is only a structural smoke check, not proof of visual equivalence.
Inspect matching fixtures and difference images before interpreting performance. A failed,
cancelled or invalidated trial must not be pooled with complete trials.

Primary upstream references:
- https://github.com/linebender/vello/blob/main/README.md (renderer-family guidance)
- https://docs.rs/crate/vello_gpu/0.3.0 (pinned API/license: Apache-2.0 OR MIT)
- https://docs.rs/crate/skia-safe/0.153.3 (pinned binding API/license: MIT)
- https://github.com/rust-skia/skia-binaries/releases/tag/0.153.3 (official binaries)
