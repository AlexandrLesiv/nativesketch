# Task 020 verification protocol

This protocol accompanies the [macOS integration contract](skia-macos-contract.md).
It defines evidence to collect, not results. Physical pen-to-screen measurements remain
pending a suitable camera. Software replay is not physical Pro Pen 3 evidence.

## Background checks

1. Core scene fixtures: cached and uncached geometry, live/predicted cancellation,
   atomic commit, original transform order, undo/delete/replacement, conservative culling,
   negative/large coordinates, retention eviction and explicit output limits.
2. Skia CPU raster fixtures: exact cached/reference comparison using identical geometry,
   canvas, paint and viewport. Cover translucent self-overlap/crossings, live commit,
   transformed middle strokes, cache reset, clipping, zoom and backing density.
3. Native boundary: actual compiler/link consumer, ownership and stale-generation rejection,
   malformed dimensions/paint/request, output unchanged on failure and no editor mutation.
4. Existing complete integration check and background XCTest preparation. Build the actual
   Metal application without starting it. Preserve exact source and binary identities.

CPU raster timings compare only the effect of caching in that CPU workload. They do not
compare Core Graphics with Metal or prove a GPU latency improvement. Use a release build,
the same generated scene/input for every mode, fixed dimensions and cache limits, warm-up,
repetitions, raw per-frame samples and separate cold/warm/live/commit/transform phases.
Report sample counts and p50/p95/p99 with preparation and submission separated. Record
cache hits/misses/bytes, toolchain, machine and flags. Retain failures; never remove slow
valid frames. Initial results establish a baseline, not an invented universal performance gate.

## Reserved desktop checks

Only after owner approval of a time-bounded session:

- Verify the foreground console, unlocked display, visible unobscured canvas and exact app.
  Record display identity, logical/pixel canvas dimensions, refresh and backing scale,
  power/thermal context and whether the Cintiq is connected. Use one GPU workload at a time.
- Run the selected native regression journeys: drawing/save/reopen, one-operation undo,
  text-field shortcut priority, menus/controls routing, resize/deactivation cancellation,
  small dark window, inspector dismissal/focus restoration.
- Exercise visible Skia output with opacity/order, selection/focus, pan/zoom, hide/reveal,
  surface invalidation and resumed latest content. A state projection alone is not a
  rendering oracle. Capture fixtures and retain any differences for inspection.
- Have the designer inspect actual native behavior and visual output. Distinguish AX
  inspection from VoiceOver speech and simulated pointer events from physical tablet use.
- Measure the release application under identical replay/canvas/cache settings. Record
  request/sample age, acquisition delay, scene preparation, CPU submission, frames in flight,
  completion/presentation signals where available, memory and cold/after-idle/sustained tails.
  A minimized/covered/locked interval is invalid environmental evidence, not a faster frame.
  A valid performance failure remains a failure. Do not silently rerun away failures.

The software comparison measures this production Skia implementation and its uncached
reference using identical inputs. Historical Core Graphics comparisons remain historical;
they are not a current-build pass or a fair current-build performance comparison. Any new
cross-backend claim requires rebuilding the same fixture and scheduling policies for both.

## Completion and retained gaps

Record accepted test IDs in the feature registry and execution separately. The task stays
in implementation until native correctness and affected rendering measurements are ready
for owner review. A built application alone is not native acceptance. Other platforms,
physical pen latency, untested display configurations and unavailable fault/device tests
remain explicit gaps. A fresh reviewer assesses the integrated change and material repairs.
