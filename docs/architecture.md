# Architecture boundaries

## Bootstrap status

`sketch-core` is an intentionally empty, platform-independent domain crate.
`sketch-ffi` exports only an ABI-version handshake. The AppKit shell links this static
library. No drawing/document API is implied by this handshake.

## Intended ownership

| Area | Owner |
| --- | --- |
| Document, strokes, layers, stable IDs, undo semantics | Rust core |
| Sampling, smoothing, pressure curves, geometry, hit testing | Rust core |
| Scene preparation, renderer implementation, caches | Future Rust renderer module/crate |
| Window lifecycle, controls, native input capture, display scheduling | Platform adapter |
| File schema and migrations | Rust persistence layer |
| File dialogs, access grants, OS document integration | Platform adapter |

Keep document state, editor-session state, and native UI state distinct.
Rendering caches are disposable and are not the document format.

## Input and rendering contracts to design in task 001

Normalize pointer ID, event phase, monotonic timestamp, coordinate space, device type,
optional pressure/orientation, buttons, cancellation, and sample corrections.
Preserve missing capabilities as missing, rather than pretending every mouse has pressure.
Keep predicted positions temporary; never persist them as actual samples.

Store editable stroke geometry and brush parameters separately from renderer objects.
Use document-space double precision and viewport-relative GPU coordinates where appropriate.
Avoid synchronous disk work, document-wide copies, and full UI updates on the ink path.
Preserve real sample order while avoiding a backlog of stale visual frames.

## Native boundary

The canonical C header is `crates/sketch-ffi/include/sketch_ffi.h`; the Swift module map
imports that header directly. Add opaque handles, error values, and explicit buffer ownership
only when a real interface requires them. Rust panics must not unwind through FFI.
The core crate forbids unsafe Rust. Any required unsafe code is isolated in adapters with
documented ownership and lifetime assumptions.

UniFFI remains an option for richer Swift/Kotlin control APIs. Do not add both generated
bindings and hand-written wrappers for the same operation without a concrete reason.
Windows will use a C# interop layer and a native GPU presentation adapter; validate
SwapChainPanel composition rather than assuming a regular window surface is sufficient.

## Renderer decision

OPEN. Start the comparison with Skia over Metal, evaluate Vello's specific version and
renderer variant. Compare identical scenes, quality, scale, input traces, and presentation.
Choose using the app's latency, correctness, memory, and integration evidence.
Do not introduce a permanent abstraction supporting hypothetical backends before the spike.

