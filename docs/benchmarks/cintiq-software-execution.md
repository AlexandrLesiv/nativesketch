# Cintiq software comparison execution contract

Date: 2026-10-14. Plan baseline: `70c546f`.
The owner authorized implementation and software comparison on the connected Cintiq;
physical pen-to-screen collection remains pending suitable camera equipment.

## Scope refinement before collection

This execution uses generated, timestamped 240 Hz traces, not recorded Pro Pen 3 input.
It exercises identical Rust editing/geometry and native presentation on the Cintiq.
Digitizer/driver delivery, physical pen feel and photon latency are not measured.
Both adapters build replacement caches synchronously in the draw path and switch
cache/live generations atomically. Incoming input remains queued during that work.
This is a matched initial implementation; asynchronous cache replacement is deferred.
The report must retain rebuild/input backlog costs and cannot claim asynchronous
responsiveness or select a physical-latency winner from this software data.

## Declared matrix

Every row below runs both Skia/Metal and actual AppKit/Core Graphics, policies R and C,
and physical canvas sizes 1600×1000 and 2560×1600, if verified to fit on the Cintiq.
Five paired fresh-process trials per cell, each 30 measured seconds. Randomize cell
order with a recorded seed and counterbalance candidate order across pairs/cells.

| Scene | Trace | Start state |
| --- | --- | --- |
| Empty | Repeated short strokes | Warm |
| 1000 committed strokes | Long growing stroke | Warm |
| 5000 committed strokes | Repeated short strokes | Warm |
| Translucency | Repeated short strokes | Warm |
| Clipping | Repeated short strokes | Warm |
| Image | Repeated short strokes | Warm |
| 1000 committed strokes | Pan/zoom, then ink | Warm |
| Empty | Repeated short strokes | First painter use |
| 1000 committed strokes | Repeated short strokes | After idle |

Warm runs use five seconds of separate replay, reset document/live/cache state, and an
untimed initial draw to establish the declared cache-ready baseline. Retain the GPU
context; reset backend path/cache identities symmetrically. After-idle runs then wait
30 seconds without input or animation. First-use runs skip warm-up and initial drawing:
report replay/reference construction and context initialization separately, not as
part of the first-paint interval or as proof of cold-system performance.

Sustained: 1000 committed strokes, long growing stroke, C policy, three paired
180-second runs per size. This adds 12 runs to 360 primary runs: 372 total and 216
measured minutes. Warm-up, idle, startup, capture and recovery add time. Apply the same fixed
two-second recovery before every process, including the first. Do not adapt recovery
to a candidate's timing. Record thermal state rather than discarding slow valid runs.
Pilot, logging-overhead, visual and failure tests are separately identified and never
pooled into confirmatory results. Unfinished cells stay pending, not counted complete.

## Timing and acceptance

Use the common monotonic clock for scheduled sample time, native replay callback receipt
and draw start. Retain complete input-advance and delivery ledgers, including commit-only
work. Frames carry newly represented sample ranges; skipped snapshots are not lost input.
Compute scheduled-to-receipt, receipt-to-first-draw and scheduled-to-first-draw per sample,
plus latest and oldest newly represented sample ages and progress intervals. Do not sum
snapshot timing fields repeatedly across frame rows; use the advance ledger for Rust work.
Render-specific callback return, Metal completion and presentation times remain separate.

Compare per-run distributions and paired differences; expose every pair and median paired
difference. Exploratory bootstrap intervals resample pairs, not correlated frames. Five
pairs provide limited uncertainty evidence. No statistical equivalence or physical latency
winner is inferred from overlapping intervals, GPU timings or callback times.

A complete run requires exact expected sample count and final document/geometry hashes,
unchanged canvas/display identity and acceptable visibility through finalization. Store
final-state images outside timing; CG bitmap captures are painter proxies, not proof of
presented pixels. Actual running-app inspection checks native scale/orientation separately.
Record partial-overlap query coverage and unknowns; do not overstate an OS visibility flag.

Verify trace/initial-state identities within every matched pair, and final output quality
across backend/cache variants before accepting timing comparisons. Preserve invalid runs,
reasons and source/binary identities. Stop cancels acquisition and halts the batch; it must
not cause an automatic retry. Any repair changes source identity and requires rechecking
and a new frozen collection build rather than mixing revisions silently.

## Hardware preflight

Observed via native/system inventory: Apple M2 Pro, 19 GPU cores; Cintiq Pro 22,
display ID 3, 3840×2160 pixels, 1920×1080 points, nominal 120 Hz. The initial configuration
mirrors Sony display ID 2 from the Cintiq. Temporarily separate displays for acquisition,
verify actual mode/backing dimensions and restore the original mirror arrangement afterward.
Wacom Center reports 6.4.14-1; this is application inventory, not verified pen firmware.
Record runtime inventory and pacing observations with the artifacts. Physical pen model
is owner-reported Pro Pen 3, not validated by a generated replay.
