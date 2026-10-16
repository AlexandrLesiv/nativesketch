# Shared viewport precision preparation

Frozen additive contract for task020, approved by the lead on 2026-10-16; implementation
starts from `1791d1e`. Renderer-only preparation is read-only and disposable. Core owns
pure f64 geometry; native Skia and CanvasKit convert its result. No renderer dependency
enters sketch-core, and no document bytes or opcodes40–44 change.

## Coordinates and typed ownership

`PixelViewport` combines the leased document-space RenderViewport and actual integer
pixel_width/pixel_height. Both pixel dimensions must be1..16384. Document x/y/width/height/
margin are finite, width/height positive, margin nonnegative, and derived scales finite
and positive. Product zoom and bitmap allocation policies remain host-owned.

Physical mapping is `(point * stroke.scale + stroke.translation - viewport.origin) *
(pixel_dimension / viewport.dimension)`, performed entirely in f64. The ratio includes
zoom and actual backing density: viewport.width=logical_width/zoom. Fractional rounding
and asymmetric backing dimensions therefore do not rely on an assumed devicePixelRatio.
Native density previously narrowed to f32; the shared mapping intentionally retains f64.

`prepare_viewport_path(&SceneStroke, PixelViewport, PreparationLimits)` returns either
LocalGeometry or complete physical-pixel commands. The shared decision uses a4096-pixel
safe extent. Giant contours use f64 clipping and bounded cubic subdivision at1/1024
physical-pixel control-hull/chord tolerance; this is not an image-diff tolerance.
All subpaths remain one nonzero-winding fill. Pixel commands already include transform,
origin and backing mapping; consumers MUST NOT apply those transforms again. Color is
straight RGBA, unchanged. Normal paths retain existing geometry-origin localization.

Preparation is synchronous render work, never called by input append. Default limits are
262144 emitted points, subdivision depth32,8388608 work steps and64MiB aggregate vector
capacity. Scratch, polygon clipping and final command capacities are accounted together;
constant bounded stack/metadata and allocator bookkeeping are distinguished. Allocation
is fallible. Wire preparation additionally uses remaining scene-byte allowance. Failure
publishes no partial output and does not change document, history or existing scenes.

## Additive NSAP v2 layouts

Existing24-byte packet and48-byte status prefix remain. All integers little-endian;
u64 references stay exact, including high bits. No native export signature changes.

Opcode45 PrepareViewportPath body24 bytes:

| Body offset | Field |
| --- | --- |
|0|scene_ref:u64|
|8|selector:u32:0 committed,1 live,2 preview replacement|
|12|index:u32: committed descriptor index; MUST0 for overlays|
|16|pixel_width:u32|
|20|pixel_height:u32|

The selected immutable scene supplies geometry, transform and viewport. Invalid selector,
index, dimensions or missing overlay reject before preparation; no caller transform exists.
Successful reply body64 bytes (absolute offsets add72):

| Body offset | Field |
| --- | --- |
|0|mode:u32:0 LocalGeometry,1 PixelCommands|
|4|reserved:u32=0|
|8|prepared_ref:u64:0 for local mode|
|16|source_scene:u64|
|24|geometry_ref:u64|
|32|stroke_id:u64|
|40|order:u64|
|48|command_count:u64:0 for local mode|
|56|pixel_width:u32|
|60|pixel_height:u32|

Opcode46 PreparedPathPage body16: prepared_ref:u64,first:u32,limit:u32. Reply body is
count:u32,reserved:u32=0 followed by existing88-byte Path records. Limit clamps to744;
first==count is EOF, first>count is rejected, limit0 returns no records. All responses
remain <=65616 bytes. Reassemble all pages before building/publishing the complete fill.

Opcode47 ReleasePreparedPath body8: prepared_ref:u64; success has no body.

## Lifetime and capacity

At most one prepared-path lease per WireApp. Any45 while held rejects Busy, including a
local-mode request. Local mode issues no lease. Pixel mode may issue an empty output lease.
Reserve reply, identity and scratch/output allowance before publishing. The independent
immutable output survives source scene release; its provenance is descriptive, not another
retain on that source. Preparation of a released source still rejects. Global monotonic
identities reject foreign-app, wrong-kind, stale and repeated release references.

Only47 releases the prepared output. App shutdown remains Busy while it exists. Paging
never recomputes clipping and never mutates the output. Scene byte accounting includes
retained output and preparation has a peak bound including intermediate capacities.
A failed call preserves any existing lease; no implicit eviction or synthetic release.

Browser consumers page/build/release one stroke at a time into a staging frame. On failure
retain the last coherent frame. Cache keys include geometry identity, stroke transform,
leased viewport and actual backing dimensions; geometry identity alone is insufficient for
pixel commands. Normal geometry and unchanged preparation decisions remain reusable.

## Evidence obligations

RENDER-001/CACHE-001/FFI-001 cover native CPU precision parity, fractional backing ratios,
positive/negative near1e9 endpoints, enclosing/grazing giant curves, translucent single fill,
transforms, explicit memory/point/work/depth/nonfinite failures, and stable document bytes.
Actual C and Wasm consumers cover exact new response fields, paging, high-bit/foreign/stale
references, failed acquisition ownership, source release, duplicate release and shutdown.
Old protocol fixtures remain byte-identical. Browser evidence uses the actual Rust outline
and CanvasKit output. GPU/native UI/physical pen latency are separate and not claimed here.
