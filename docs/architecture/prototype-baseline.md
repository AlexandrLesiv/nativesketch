# Prototype architecture at 712059d5

Historical snapshot of the selected codex/github-pages base, not current policy.
Renderer OPEN and intended ownership below describe that base. The accepted target in
[architecture](../architecture.md) and [decision 0003](../decisions/0003-skia-and-quality-architecture.md) supersedes them.
No comparison-results implementation is imported.

## First-stroke boundary

`sketch-core` owns editable stroke samples, brush style, selection, uniform transforms,
undo/redo, and versioned local persistence. `sketch-ffi` exposes ABI 2 through an opaque,
uniquely owned editor handle. The AppKit adapter captures input, maps viewport points
into document coordinates, and paints caller-owned Rust path commands with a provisional
CoreGraphics painter. This is an interaction prototype, not the production renderer.

Handles must be used serially, input buffers are borrowed for one call, output buffers
are caller-owned, and Rust catches panics at the boundary. A panic invalidates the handle.
The canonical header documents status values and ownership. Save/load are synchronous
and run outside pointer transactions; large-document background I/O is a remaining limit.

## Portable hosts

The same Rust editor now serves AppKit/UIKit through the C header, Android through
JNI, WinUI through C# P/Invoke, GTK directly as a Rust dependency, and browser through
`sketch-wasm` protocol 1. The byte format is common across all hosts. Platforms own
viewport state and native controls; no host reimplements document/history semantics.

Android desktop adaptation shares the same Activity, JNI bridge and Rust editor.
Window width selects compact or expanded native controls; size changes cancel temporary
gestures and retain committed content, history and selection. Keyboard/mouse/stylus
adaptation and content-URI permissions remain in Kotlin. ARM64 and x86-64 package the
same Rust semantics; no separate ChromeOS or Googlebook document implementation exists.

Export returns bytes and a revision; only a confirmed successful write of that
revision may mark clean. New/import/load invalidate earlier save acknowledgements.
The browser's download-only path keeps dirty state because dispatching a download is
not confirmation of persistence. Android uses Storage Access Framework Save a copy;
file-provider partial-new-file failure cannot universally be made atomic. Desktop
path writes use the core's same-directory atomic replacement.

Native painters remain disposable geometry consumers. GTK and browser packages build
separately, keeping GUI dependencies outside the core workspace. C ABI 2 ownership
and raw Wasm token lifetimes are documented in the header and `crates/sketch-wasm/WASM.md`.

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

## Input and rendering contracts

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


See [first-stroke interactions](../design/first-stroke.md) and the
[provisional presentation decision](../decisions/0002-first-stroke-rendering.md).

## Apple device capabilities

Mac, iPhone/iPad and visionOS expose editing. tvOS and watchOS expose a private Swift
viewer adapter over the same Rust ABI, limited to validated imports, state and path
copies. Native viewport changes do not mutate document bytes. Vision is a native 2D
UIKit window; no spatial document schema or immersive renderer is introduced.

TV downloads explicit HTTPS links with a streaming limit and imports a candidate before
replacing the cached copy. Watch receives an explicit iPhone snapshot via WatchConnectivity;
metadata and bytes share an atomic cache envelope. Receiving validates in a separate Rust
handle before replacing presentation. Sending a copy never acknowledges a document save.
Watch runs independently with optional phone integration, using stable Rust ARM64 builds;
arm64_32 is not configured. Native source adapters prefer iPhone on explicit refresh, then
use a configured server for the same published identity. Phone-only copies cannot fall
back to unpublished cloud data. Rust source policy compares namespace/document identity,
immutable revision/parent, server generation and content hash; timestamps do not order
copies. Swift validates hashes, correlates requests, bounds transport and atomically
publishes Rust-validated cache entries. Newer metadata advances even for identical bytes.
The production network/authentication service is not configured; current network adapter
supports controlled public/developer sources. See `apps/watchos/SOURCES.md`.
