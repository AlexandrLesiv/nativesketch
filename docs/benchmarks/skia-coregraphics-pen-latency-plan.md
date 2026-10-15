# Skia versus Core Graphics: physical pen-to-screen comparison plan

Status: software comparison complete (368 accepted runs and four classified sustained failures; 372 outcomes). The authorized
software scope and frozen matrix are specified in
[cintiq-software-execution.md](cintiq-software-execution.md). Physical measurements
remain pending suitable camera equipment and the real-pen input adapter/validation.
Date: 2026-10-14. Planning baseline: `37e717b4aa08bc92fbc5edb8100112222b135aa4`.
Follow-up to [task 002](../../tasks/002-renderer-comparison.md).

## Decision to make

Which macOS pipeline gives our drawing app the most responsive, correct visible ink:
Skia Ganesh over Metal, or an AppKit canvas painted with Core Graphics?

Compare complete usable pipelines, including their native presentation costs. Previous
Skia submission times and Core Graphics draw-callback times have different endpoints;
neither establishes which shows ink sooner. They guide workloads, not the winner.
Vello is outside this experiment. Preserve its historical code and results. Production
selection remains OPEN, with owner acceptance before permanent integration or merge.

The owner has made a connected **Wacom Cintiq and Wacom Pro Pen 3** available. These are
the physical input and display for this experiment. Execution verified a Cintiq Pro 22 at 3840×2160 physical pixels, 1920×1080 points and nominal 120 Hz on the M2 Pro Mac. Wacom Center reports 6.4.14-1; the driver/pen firmware and physical Pro Pen 3 delivery remain unverified. Physical acquisition is explicitly deferred until suitable camera equipment is available. See the execution contract and completed software report for current evidence.

## 1. Two complementary kinds of evidence

| Track | Input | What it answers | What it cannot establish |
| --- | --- | --- | --- |
| Deterministic replay | Identical timestamped samples, pressure and viewport trace, delivered at the same wall-clock offsets | Equivalent work, correctness, scheduling, cache behavior, software-stage costs under identical input | Physical digitizer-to-visible-pixel latency |
| Physical pen | Pro Pen 3 on the Cintiq, filmed with the displayed ink in the same recording | Contact-to-first-ink and motion-to-visible-ink distributions for the complete pipeline | Exactly identical hand motion across backend trials |

Capture a synthetic drawing made with the real pen for replay, with no personal document
content. Replay advances by monotonic elapsed time, not by completed frame count. Slow
rendering must not slow the input generator and thereby hide backlog. Record delivery
jitter and event/sample counts. Keep all received real samples for Rust geometry; a
renderer may skip stale intermediate snapshots, never discard geometry to look faster.

Physical trials use repeated, counterbalanced movements. An actuator could improve
repeatability if one is available, but is not required or assumed. Never describe manual
A/B movements as identical samples. Record speed, direction, pressure and location and
report imbalance; use prescribed ranges fixed in the pilot rather than selecting only
favorable movements afterward.

## 2. Freeze the shared contract before coding

One lead-owned, versioned experiment contract defines these boundaries. Extend the
isolated experiment, not the production document format or a permanent renderer abstraction.
Reuse task 003's normalized-input conventions; implement only the recorder/replay subset
needed here if task 003 has not landed, without claiming that whole task complete.

| Boundary | Identical for both candidates |
| --- | --- |
| Native input | Same AppKit event handler, pen identification, coordinate mapping, driver settings and delivery policy |
| Samples | Sequence ID, native timestamp and clock domain, callback receipt time, phase, position, pressure/capability flags, cancellation and viewport revision |
| Rust editing | Same editor revision, brush, pressure curve, smoothing, outline generation, clipping and ordering |
| Render snapshot | Immutable document revision, live-stroke revision, last included real sample ID, viewport revision and geometry hash |
| Rendering intent | Same visible objects, outlines, fill rule, colors, alpha, clip geometry, decoded images and logical draw order |
| Viewport | Same document transform, physical pixel size, backing scale, Cintiq screen location and color-space intent |
| Caching | Same policy, cache keys, invalidation events, resolution, coverage and application-controlled budget |
| Scheduling | Same update trigger, one pending application redraw request, newest complete snapshot when rendering starts |

Disable application prediction for the primary comparison. Keep smoothing fixed and
identical; do not change the pressure law between backends. Record driver-side filtering
or prediction if exposed; unknown device processing remains an explicit limitation.
Additional predicted samples, if tested later, stay temporary and form a separate matched
experiment with overshoot/error measurements. They cannot replace the primary results.

Pin Skia, dependencies, compiler, SDK, optimization and linker settings in a manifest.
Start with the already tested Skia version unless a documented compatibility issue
requires changing it before the pilot. Use release builds without debuggers attached.
Both candidates include the same Rust work in the same host and differ at the painter /
native presentation boundary. Backend-specific path conversion is included in its costs.

## 3. Use real native presentation and the same canvas

Use one AppKit experiment host with a common input receiver and interchangeable painters:

- **Skia:** Ganesh/Metal rendering to a native Metal-backed canvas. Prefer direct Metal
  integration without the old comparison harness's wgpu/offscreen preview scaling.
  Validate texture ownership, flush/submit ordering and drawable lifetime before timing.
- **Core Graphics:** actual layer-backed `NSView.draw(_:)` using AppKit's current
  `CGContext`. Do not substitute a CPU bitmap uploaded to a Metal canvas. Bitmap caches
  for static content are allowed only under the shared cache policy below.

Use one candidate at a time, in the same window position on the Cintiq, without mirroring
or scaled previews. Verify the view's actual backing dimensions and Skia drawable
size. Start with **1600 × 1000 physical pixels** if it fits completely on the Cintiq;
add **2560 × 1600** only if it fits without scaling. If either cannot fit, freeze a
smaller common size before the pilot and state why. Never crop or resize one candidate
alone. Record window points, backing scale, active display mode and measured refresh.

Set the same sRGB intent, background and opacity, with no application animations or
extra live stroke view exclusive to one backend. Validate antialiasing and edge quality;
the same nominal sample count is not sufficient evidence of equal quality.

Primary scheduling is event-driven invalidation with no intentional extra display-link
wait. Submit the newest snapshot once the adapter can accept work, with no application
FIFO of stale frames and no busy-loop immediate redraws. Retain normal synchronized
native presentation. Record Metal drawable count, synchronization settings and actual
backpressure. AppKit/Core Animation's internal queue is not directly equivalent to a
Metal queue; do not claim identical internal frame depth or force a GPU completion wait
into just one adapter. Native scheduling differences are part of the pipeline result.
A separately labeled matched display-paced diagnostic is allowed if needed to locate
a delay, but cannot silently replace the primary configuration.

## 4. Match caching explicitly

Run two named policies; compare candidates only within the same policy.

**R — retained vectors, full redraw (control).** Retain immutable paths and decoded
images; redraw the complete visible scene plus current stroke on each update. No
application raster cache of the final scene. Share visible-object culling and redraw
extent. OS and driver caches remain implementation details and must be acknowledged.

**C — cached static background plus fresh live ink (primary product candidate).** At a
fixed viewport, cache committed content at the actual canvas resolution. Both adapters
use the same logical cache coverage, key `(document revision, viewport transform,
backing size, color configuration)` and invalidation rules. Composite that background
and the current Rust live-stroke outline into one frame. Do not cache an old live stroke
instead of applying new samples. The cache may be a Skia texture or Core Graphics image;
its creation, upload, memory and composition costs belong to the corresponding pipeline.

For this bounded experiment use a full-viewport background cache rather than two
independently designed tile systems. Use the same number of logical cache generations
(active plus replacement), visible extent and pixel format intent; report actual bytes
and unavoidable backend overhead separately. No unlimited hidden application cache.
Do not require identical internal library caches, which cannot reliably be enforced.

On pen-up, retain the final live outline over the old background until a replacement
background including that stroke is ready. Switch the background and remove the live
outline in the same displayed generation: no gap, duplicate darkening or stale revision.
New input must continue while a replacement is pending. Tag snapshots and cache generations
so recording can verify the handoff. Include rebuild work even if it occurs asynchronously.
Use the same logical work and scheduling policy; do not block one input handler while
moving the other's work off-thread. Validate safe ownership and thread constraints.

Pan, zoom, resize, undo/redo and document changes invalidate the same cache keys. During
viewport changes, use full vector redraw until the final viewport cache is ready for
both backends. Do not stretch a stale raster for just one candidate. Tiled infinite-canvas
caching is a later optimization: this experiment tests viewport rendering and transition
costs, not unlimited document scalability.

## 5. Measure physical pen-to-screen latency

### Equipment and calibration gate

Verify the Cintiq model, resolution, refresh mode, cable/connection, Wacom driver version,
pen identification and pressure reports. Fix the same mapping, pressure settings,
display processing and power configuration across pairs. Record the Mac/OS/GPU/RAM and
all other connected displays. Do not infer refresh from a marketing specification or
from the older SONY display measurements.

Use a tripod-mounted high-speed camera seeing **both the physical pen tip and actual
ink**. Prefer 1000 fps or better for close differences; 240 fps can support an initial
coarse comparison if exposure and image detail are adequate. At 240 fps one camera
frame is about 4.17 ms; at 1000 fps it is 1 ms. These are sample intervals, not guaranteed
measurement accuracy. Determine actual frame timestamps, exposure, dropped/duplicated
frames, rolling-shutter behavior, display scan direction and pixel response.

Keep contact and measured pixels near the same camera row where practical, calibrate
remaining row offsets and report uncertainty. Use a visible tip and an offset filming
angle that does not hide the first ink under the nib. Freeze a luminance/change threshold
for ink detection using blank/static controls. If contact or first ink is obscured,
mark the trial unscorable with its reason; do not guess the missing time.

The camera is the common clock for physical motion and visible response. Software clock
alignment is optional diagnostic evidence requiring calibration, not a prerequisite for
optical latency. A render-frame ID patch may aid association only when drawn with the
same snapshot and frame as the ink; the patch is not the latency endpoint and must not
be an independent faster overlay. Measure the actual stroke pixels.

If no suitable camera is available, proceed only with implementation and software replay
when execution is authorized. Report physical latency as **not measured** and leave the
latency decision open. Screen recording, an OS cursor, GPU completion and presentation
callbacks do not substitute for filming the physical pen and emitted pixels. No equipment
purchase is included in this plan.

### Primary outcomes

| Metric | Physical start | Visible endpoint / analysis |
| --- | --- | --- |
| First-mark latency | Nib contact with surface, with contact uncertainty annotated | First qualifying stroke pixels, not cursor or marker |
| Continuous-ink latency | Pen crossing a calibrated position while already drawing | Recovered rendered centerline/sample endpoint reaches that position; compare time-shifted trajectories |
| Commit handoff | Pen lift / final motion and last real sample | Final stroke appearance and uninterrupted handoff to cached content |
| Visible stalls | Continued physical motion | Duration and frequency of the recovered ink endpoint not advancing; report per trial/minute |

Use straight, monotonic constant-speed sweeps for primary motion estimates. Map camera
coordinates to display coordinates and compare position-crossing times. Calibrate static
nib-to-rendered-centerline registration at each tested location and pen tilt, including
camera parallax and display mapping. Balance opposite-direction sweeps and report
residual spatial uncertainty separately; a fixed position offset must not become an
unexplained time delay. Use a fixed-width
diagnostic brush for primary sweeps and recover the rendered centerline endpoint from
its known cap geometry. The outer painted frontier leads the last sample by the cap
radius: treating that frontier as the sample creates a speed-dependent latency bias.
Calibrate endpoint extraction across the declared speeds using known geometry and
include cap, antialiasing and detection-threshold uncertainty. If endpoint recovery is
not reliable, the sweep is unscorable, not a zero-latency result. Validate representative
pressure-varying brushes separately as secondary drawing cases. Do not divide an
arbitrary gap by nominal speed during acceleration or turns. Corners, reversals and
pressure changes are separate accuracy/lag cases; ambiguous correspondence is not a
made-up latency sample. Contact-to-first-ink includes pen activation threshold and device
sampling; it is not purely renderer cost. Pressure and approach speed are controlled and
reported, not subtracted without an independent measurement.

For every optical event retain the last-negative/first-positive frame bounds. Derive a
latency interval from both endpoints and calibration uncertainty, retaining the interval
in analysis rather than claiming sub-frame precision. Validate extraction against a
blinded manual annotation subset. Report top/center/bottom screen locations separately
when checking scanout effects, keeping paired trials at matching positions.

### Software diagnostics alongside video

Log native event time, callback receipt, Rust apply/geometry start/end, snapshot/sample
IDs, redraw request/start, cache work, painter calls and CPU submission/return. Record
Metal acquisition, completion and available presentation timestamps separately. Include
event backlog, skipped snapshots, represented-sample age and memory high-water marks.
Map clocks explicitly; do not subtract timestamps from unrelated clocks.

Core Graphics callback return is not raster completion or photon time. Metal presentation
time is useful corroboration, not a substitute for physical measurement. Missing callback
samples stay missing. Do not infer presented-frame misses from CPU duration alone, or
sum overlapping CPU/GPU spans. Buffer logs in memory and write outside timed trials;
measure instrumentation overhead with matched logging-enabled/disabled pilot pairs.

## 6. Workloads and trial design

First run a small correctness/calibration pilot; its data is labeled exploratory and
excluded from the confirmatory decision. Freeze the manifest, analysis, numerical
practical-equivalence margins and trial counts after that pilot, before collecting
confirmatory data. Changes afterward require a new version and new matched trials.

**Primary physical matrix:** two painters × policies R/C × two backgrounds (empty and
1000 committed strokes) × one verified canvas size. Use the same brush, scene hashes
and target paths. For each condition collect first-mark contacts and slow/fast straight
sweeps. Start planning at five paired blocks, each with 20 contacts and 10 sweeps per
candidate/condition: 100 contacts and 50 sweeps per candidate/condition. Split across
at least two sessions. This is a pilot-adjustable workload estimate, not proof of
statistical power. Camera handling and annotation dominate elapsed time; estimate the
required host reservation after the pilot, before the main runs.

Randomize balanced backend order within condition blocks; counterbalance condition order
and publish the seed/order before collection. Allow equal warm-up, cache-ready checks
and recovery intervals. Use anonymous A/B labels for the operator and video annotator
where practical; preserve the decoding key separately until extraction is frozen.
Do not alternate by running both canvases simultaneously on different screen regions.

**Stress and transition matrix:** 5000-stroke background, translucency/clipping/image
fixtures, repeated short strokes/commit handoffs, long growing stroke, pan/zoom then ink,
first stroke after process launch and after idle. Cover both policies and both painters
in deterministic replay. Add the second canvas size when supported. Use five paired
30-second replays per declared cell and three paired 180-second continuous-ink trials
for the 1000-stroke cached case. First-use trials have no artificial warm-up; report them
separately from warmed results. Never pool resolutions or cache policies.

Optically test commit handoff, corners/pressure, and at least the 5000-stroke cached
condition as predefined secondary cases. Set their counts in the pilot manifest. If
replay reveals another failure, report it and add a symmetric, explicitly exploratory
physical follow-up rather than quietly changing the primary test set.

Validate cancellation, pen leaving range, missing pressure, undo/redo, viewport changes
and cache resource recovery outside timing. Compare final Rust document/geometry hashes
for identical replay and capture matched output images. Inspect edge quality, opacity,
pressure width, corners and final endpoints at full resolution. No winner may achieve
low latency by omitting samples, shortening strokes or reducing fidelity.

## 7. Analysis and decision rules

Analyze trials and paired blocks, not millions of correlated video frames as independent
observations. Publish per-condition p50/p95, worst observed stalls, count/duration of
trials and paired differences with 95% confidence intervals using resampling at the
paired block/session level. Preserve temporal correlation within sweeps. With only a
small number of sessions, show their separate results and label uncertainty accordingly.
P99 is exploratory unless the pilot justifies enough independent tail observations;
100 contacts alone cannot give a stable p99 estimate.

Before confirmatory collection, define a practical margin in milliseconds for first-mark
and continuous-ink latency, plus allowed regression margins for tails and quality. Use
measurement resolution, pilot repeatability and the product's responsiveness needs to
choose these margins, with backend labels masked. They are proposed decision criteria,
not previously verified product budgets. Do not choose margins from which backend wins.
Include optical uncertainty in confidence/sensitivity analysis; if it dominates the
margin, improve the equipment or leave the result unresolved.

The cached policy is the primary product choice; full-redraw results explain sensitivity.
A latency winner requires a paired improvement exceeding the practical margin with
uncertainty accounted for, without a material regression in the other primary latency
outcome, dense-case tails, commit behavior or quality. Define all primary comparisons
and use simultaneous intervals or another preregistered multiplicity correction;
do not cherry-pick whichever scene happens to favor a backend.

Possible recommendations:

- **Skia:** a meaningful, repeatable visible-ink advantage with acceptable quality, memory
  and build/distribution burden.
- **Core Graphics:** a meaningful advantage, or demonstrated practical equivalence with
  preferable macOS integration/footprint for this product.
- **Tradeoff / inconclusive:** mixed workloads, intervals too wide, missing physical
  evidence or quality failures. An interval crossing zero alone does not prove equivalence.

Report memory, first-use costs and size separately as secondary considerations. Prior
bundle measurements used a different harness; do not label them the exact size of this
new host. Results apply to the measured Mac/Cintiq configuration and macOS pipeline,
not automatically to iPad, Android, Windows or web. A macOS-specific winner need not
be the best shared renderer for every platform.

## 8. Host control, invalid trials and evidence

Only one benchmark/automation owner uses the GPU and native UI during acquisition.
No concurrent builds, profiling sessions or other benchmark windows. Record thermal and
power state before/after blocks, and apply the same cooldown rule. Use scoped activity
assertions during an authorized run; do not change global sleep/security settings.

Keep the canvas fully visible, unchanged and on the selected Cintiq. Record visibility,
screen identity, active mode, backing size, window geometry, session lock/sleep and Stop
through finalization. A move, resize, obscuring window, display disconnect/mode change,
lock or interruption invalidates the affected trial. Preserve raw output and reason;
rerun its matched pair under the frozen rule, never discard a slow valid trial. Pen
trajectory/optical exclusions are separately counted, with backend-specific exclusion
rates visible. Stop always works and does not silently schedule a retry.

Preserve exact source and binary hashes, lockfiles/build flags, hardware manifest, input
traces, cache/snapshot logs, raw timings, videos/annotations, calibration, exclusions,
randomization seed and analysis version. Produce machine-readable summaries plus an HTML
comparison with latency distributions, paired differences/uncertainty, cache-transition
examples and a clear recommendation. Label measured, derived and unsupported metrics.

The previously stopped 15 bundle-size rendering checks remain pending. This plan neither
resumes them nor converts their partial coverage into a completed result.

## 9. Implementation sequence and ownership

| Phase | Owner and paths / responsibility | Exit gate |
| --- | --- | --- |
| Contract + pilot design | Lead owns this plan, manifest/ABI; ui_ux_designer reviews physical procedure and visible correctness | Hardware inventory, shared contracts and primary outcomes agreed |
| Input/snapshots | rust_engine, isolated worktree; experiment input/replay and shared snapshot/cache-state logic | Deterministic sample/geometry hashes; cancellation, ordering and commit generation tests |
| Native host/painters | macos_ui with rendering_performance consultation, separate worktree; experiment Swift host and painter integration | Actual Cintiq presentation, common scheduling, both painters correct; no permanent production replacement |
| Collection/analysis tools | build_reliability after one implementation writer finishes; scripts and analysis only | Manifest validation, missing-data handling, synthetic timing/annotation checks |
| Calibration pilot | Lead or designated rendering specialist, exclusive host | Camera uncertainty, logging overhead, feasibility and frozen confirmatory protocol |
| Main acquisition | Same designated host owner | All declared paired cells complete or explicitly pending; no hidden exclusions |
| Assessment | Designer checks implemented drawing behavior; fresh reviewer audits integrated code, raw evidence and interpretation | Findings resolved or limitations explicit |
| Decision report | Lead | HTML comparison, reproduction instructions and proposal for owner acceptance; no merge |

At most two implementation writers and three subagents. Agree input/cache/FFI contracts
before writes; identify starting commit, owned paths, dependencies, acceptance cases and
stop condition for each handoff. Serialize shared contract edits and all native runtime
work. After two unproductive repairs, diagnose or explicitly reduce scope before retrying.

During implementation run focused replay/cache/lifetime tests, Swift/Rust integration,
visual checks and `./scripts/check` for integration changes. Recheck the integrated
revision, not just worker branches. This document-only planning change requires link/diff
validation and independent methodological review; it does not require running the app.

## References and limits

- [Apple: NSView.draw(_:)](https://developer.apple.com/documentation/appkit/nsview/draw(_:))
  defines the AppKit-provided drawing context and dirty-region contract.
- [Apple: MTLDrawable.presentedTime](https://developer.apple.com/documentation/metal/mtldrawable/presentedtime)
  describes reported display time and the zero value for unpresented/dropped drawables.
  This remains a software observation, not optical measurement of the pen and pixels.
- [Skia: SkCanvas creation](https://skia.org/docs/user/api/skcanvas_creation/)
  distinguishes raster and Ganesh surfaces; this plan tests Ganesh/Metal.
- [Microsoft: touch-down hardware latency methodology](https://learn.microsoft.com/en-us/windows-hardware/test/hlk/testref/overview-of-measuring-touch-down-hardware-latency)
  illustrates the need for a defined physical contact start. Its endpoint is OS input
  receipt, so its hardware-only numbers must not be called our pen-to-screen latency.

The optical procedure, trial counts and decision rules above are this project's proposed
protocol. They are not claims of an already calibrated instrument or completed study.
