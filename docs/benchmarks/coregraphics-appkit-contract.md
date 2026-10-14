# Core Graphics / AppKit benchmark extension

Status: implementation contract, 2026-10-14. Baseline e0c7f99. Production selection OPEN.

This is a separately labeled experiment of actual layer-backed NSView.draw with
NSGraphicsContext.current.cgContext. Do not substitute a bitmap CGContext uploaded to
Metal. Existing GPU evidence is historical context, not an identical presentation path.

## Frozen scope

Pilot: all eight fixtures at 1600x1000, first-use + 12 warm-up + 30 measured frames,
separate from primary evidence. Quality: frame 599 at each requested resolution,
matching original completion repeat 0. Native visible inspection supplements untimed
bitmap/cacheDisplay captures; captures are not proof of exact presented pixels.

Primary: eight existing fixtures x 1600x1000 and 2560x1600 x three fresh-process repeats,
48 trials. One first-use frame + 120 warm-up + 600 measured frames. Mode appkit-paced
only. Sustained: panzoom 1600x1000, three fresh processes, 120 warm-up + 180 measured
seconds. No completion-mode matrix or current Metal rerun is implied. Original GPU
measurements retain their original sources, conditions and definitions. Report CG vs
GPU as distinct pipeline costs, never a causal overall speedup or pen-latency ranking.

Pilot gates quality, dimensions and likely runtime before expansion. Initial automatic
measurement window cap 90 minutes; preserve incomplete coverage if cap expires. Do not
reduce frame counts or omit expensive workloads silently. No simultaneous agent builds,
GPU work or native UI automation during timed trials. Scoped activity/display inhibition;
floating automated window, main-thread lifecycle checks, invalidation on minimize,
full occlusion, display/backing/size/session/sleep changes, and authoritative final host
status through output. All attempts and failure reasons retained.

## Fixture boundary

Production FFI unchanged. New experiment native_fixture.rs exports borrowed POD from
existing fixtures::Fixture; core generation, input traces and identities unchanged.
Header owner is the lead; implementation worker may propose changes before applying.
All calls serialized on main thread; caller owns opaque handle; borrowed buffers valid
until next prepare/destroy. Rust catches panics; a failing/panicked fixture must not be
reused except destruction. Null/malformed arguments fail. rc_error(NULL) supplies error.

RcCommand: u32 verb (0 move,1 line,2 cubic,3 close), u32 reserved, double points[6].
RcOutline: u64 stroke_id, command_start, command_count; double rgba[4].
RcFixtureFrame: command pointer/count; outline pointer/count; clip command pointer/count;
u32 has_clip, geometry_changed; double transform[6], image_transform[6]; double
input_ms, outline_ms, export_ms. Export flattened arrays only when geometry changes.
Image bytes are identical straight RGBA 2048x2048; image drawn only for image fixture.
Expose create(name,width,height), prepare(handle,frame,out), metadata_json(handle),
state_hash(handle,frame,actual_flag), image_rgba(handle,out_byte_count), destroy(handle).
Metadata and identity work outside timed service. State hashes use existing definitions;
final actual geometry hash must equal expected. Metadata includes spec/input/image hashes,
initialization/identity-precompute facts and exact curved clip identity. Reuse the exact
kurbo ellipse path at tolerance0.05 through a common fixture helper; no addEllipse shortcut.
Phase indices reset to zero, including live sample state. Capture the actual final measured
index: primary 599, pilot 29, sustained measured-count minus one.

## Native drawing and dimensions

Canvas point dimensions = target physical dimensions / backing scale. Verify
convertToBacking(bounds) and pixel alignment; record the actual CGContext CTM separately.
Native layer-backed AppKit drawing can expose an identity CTM for a 2x backing view:
Core Graphics may record drawing for later execution. Do not infer backing resolution
from CGContext CTM or require its scale to equal the window backing scale. Validate a
finite, invertible, axis-aligned context and verify actual capture dimensions and content.
Require full canvas/window to fit visible screen and fail unsupported size rather than
silently downscale. Clear white and rasterize fresh each benchmark draw. Cache CGPaths
for retained geometry; rebuild live paths when geometry_changed. Nonzero fill once per
whole stroke; source-over, explicit sRGB colors/image, antialiasing enabled. Image .low
interpolation is requested bilinear behavior, not a guarantee of equal filter output.
Record color/premultiplication/CTM/image orientation and verify quality before conclusions.

Display-link opportunities enqueue at most one pending main-thread request. Advance the
frame-indexed trace only on actual benchmark draw callbacks; count/coalesce opportunities
rather than accumulate stale requests. Record actual callback/request count and scheduler
policy. No fixed promise that AppKit redraw or GPU presentation occurs on every pulse.

## Output and timing schema (separate from GPU schema 1)

manifest.json schema_version=2, experiment=coregraphics-appkit-v1, backend=coregraphics,
mode=appkit-paced. Include status, source_revision/source_diff_hash/build identities,
fixture/width/height/repeat_id/warmup/frames_requested or duration_seconds, fixture hashes,
quality policy, physical canvas/drawing context, display/OS/hardware, activity/lifecycle,
first-use/init, measured counts/elapsed, scheduling counts, final expected/actual hashes,
OS peak RSS (whole-process incl linked libraries/instrumentation/untimed output).

frames.csv: phase,index,state_hash,request_s,draw_start_s,draw_end_s,input_ms,outline_ms,
export_ms,path_ms,draw_ms,service_ms. All clocks monotonic CACurrentMediaTime-compatible;
Rust stage durations are Instant durations. service_ms is draw callback entry through
fixture preparation, path construction and Core Graphics paint calls (before identity
bookkeeping); draw_ms is paint body alone, nested in service. request-to-draw and CPU
callback-start intervals are separate diagnostics, not physical input latency or FPS.
First-use/warm-up/measured phases separate; retain per-process p50/p95/p99/max, use median
of three process p95s and min-max range, never pool all frames. No confidence intervals.

Record measured_start_s at the first measured callback start and measured_end_s after
final measured bookkeeping; elapsed must equal their difference. Validate service
against its timestamp span and verify PNG integrity, not just its signature. Explicit
validation-only finalization-delay runs cannot enter performance evidence.

GPU time/completion, presentation timestamps, missed-display-deadline rates, GPU peak memory,
physical footprint and physical pen latency are unsupported, not zero. Native draw return
can exclude deferred composition; do not sum with old Metal GPU spans or call equivalent.

host-status.json authoritative: status/reason, lifecycle_start/end/events, activity_assertion
(including active through output), output validation and final dimensions. Complete manifest
alone cannot admit a late-invalidated trial. Finalization yields to the run loop between
capture, writes and authoritative status; interruption history is latched through output. Invalid/cancelled CLI exits nonzero.
final.png + readback.json: untimed capture provenance, dimensions, RGBA/content hashes and
state/fixture identity. Separate visual correctness evidence from the timed AppKit context.

## Ownership

Renderer worker: native fixture FFI + tests, shared clip helper, separate Swift CG host,
build-coregraphics-host + wrapper. No production app changes. Tooling worker: separate
run/analyze-coregraphics-comparison scripts and failure tests; no changes to old data/exporters.
Lead: this contract/header, integration, serial runtime checks, evidence/report. Fresh
review and designer quality review before final recommendation.
