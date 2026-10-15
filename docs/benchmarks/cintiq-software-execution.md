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

## Recorded continuation amendment

See [bounded performance failures](cintiq-software-protocol-amendment.md) for the
2026-10-15 opt-in collector amendment, diagnostic evidence, trace limitations and
separate native/collector provenance. Native measurements and watchdogs stay unchanged.

## Completed acquisition record (2026-10-15)

The [HTML/SVG report](../evidence/cintiq-software-comparison.html) and
[validated analysis](../evidence/cintiq-software-analysis.json) contain all 372 outcomes:
368 accepted, four classified Core Graphics sustained failures, zero pending and zero
invalid entries in the retained matrix. All 186 pairs have matching identities;
182 support numeric paired differences. All 360 primary runs completed.
Skia completed six of six sustained trials; Core Graphics completed two and failed four.
Accepted means complete validated artifacts, not a deadline guarantee: the cooperative
main-thread watchdog cannot preempt a long callback. The two completed Core Graphics
sustained runs still have per-run p95 input-to-draw-start delays of 14.46 and 21.19 seconds.

Native source remains `8a26d1d7a25bfcbc526584d386a4f49c30ecbbcf`; binary SHA-256
`9e7ad2880182ab4ea5416cff763f417fde59a31dab4a992d1d0faad27cb2c16d`.
Final collection: `artifacts/cintiq-software-8a26d1d-session8`, frozen at collector
revision `1dcdb92` with runner/analyzer hashes unchanged from `acabb64`.
Earlier outcomes retain their original receipt and recovery provenance. All retained
initial/final power observations report low-power mode false and thermal state 0.

All eight acquisition sessions remain preserved. Six environmental attempts were
rejected: Dock overlay (session1), Sony display change (session3), Window Server
overlay (session4), Dock overlay (session5), locked startup before manifest creation
(session6), and owner-confirmed covered window/desktop switch (session7). Session2
stopped on the first performance failure before the documented continuation amendment.
An accepted but unpaired Skia session3 run remains excluded; both pair members were
repeated together. No performance failure was retried. Session8 completed acquisition
at 07:25 UTC on 15 October. The original Sony-to-Cintiq mirror was restored afterward,
and the task's temporary awake assertion was stopped.

The phrase “long growing stroke” means accumulated outline geometry: the trace
retraces the same curve with abrupt position resets. It is not smooth spatially growing
physical pen motion. The report separates these adversarial stress cells from short
strokes. Its recommendation prioritizes Skia for the next implementation, with long-stroke
and cache optimization still needed. Core Graphics remains a lightweight baseline.
Physical-latency ranking and permanent production selection remain open.

Secondary [first-mark evidence](../evidence/cintiq-first-mark-analysis.json) covers
all 80 accepted first-use/after-idle runs. It follows the first expected sample through
its actual delivery batch to its first representing draw start, verifies immutable
receipts, and binds the output to the analysis and collection checksums. First callback
wall time remains separate; host-start time begins at the first Swift statement, includes the explicit one-second
autostart wait and preparation, and, after idle,
warm-up plus the idle wait. No primary metric or threshold changed.

Reproduction after acquiring the local raw artifacts:

```sh
./scripts/analyze-cintiq-comparison artifacts/cintiq-software-8a26d1d-session8 \
  --output docs/evidence/cintiq-software-analysis.json
./scripts/extract-cintiq-first-mark docs/evidence/cintiq-software-analysis.json \
  artifacts/cintiq-software-8a26d1d-session8 \
  --output docs/evidence/cintiq-first-mark-analysis.json
./scripts/build-cintiq-report docs/evidence/cintiq-software-analysis.json \
  --context docs/evidence/cintiq-software-context.json \
  --first-mark docs/evidence/cintiq-first-mark-analysis.json \
  --output docs/evidence/cintiq-software-comparison.html
```

Only the reviewed narrative is supplied by the context file; it does not change
measurements. Native integration checks and preflight evidence remain tied to their
original revisions. Final portable tool tests and independent review are recorded below.
Physical measurements, real Wacom input characterization, and the previously stopped
15 bundle-size rendering checks remain pending.

### Final verification

Report/tool revision `152ed43` passed 49 analyzer/report tests, 12 first-mark/report
tests, task-board validation (14 tasks), and whitespace checks. The earlier full
integration check, release build, 20 experiment tests and strict Clippy remain tied
to unchanged native revision `8a26d1d`; unrelated native/platform suites were not
rerun for the report-only integration.

The independent reviewer reproduced the final analysis exactly, checked every carried
artifact and recovery checksum in sessions5–8, checked all80 first-mark rows against
raw records, and independently ran the61 portable tests. No blocking finding remains.
The designer reviewed final HTML/source and the supplied first-mark screenshot; wording
and header-wrapping findings were repaired. The lead inspected the browser at desktop
and390px widths, checked22 SVG charts and bounded horizontal scrolling, and reset the
viewport override. Review screenshots and checksums are retained under
`artifacts/cintiq-final-review-152ed43/verification.json`.

No new native/physical automation was performed during final report review. Earlier
native inspection remains scoped to the recorded controls and screenshots; dark
appearance, full keyboard traversal, physical pen feel and temporal cache-handoff
appearance remain unverified. No merge into main or permanent backend selection occurred.
