# 002: Choose the initial renderer using evidence

Status: active — owner authorized execution of the detailed plan on 2026-10-14. Roles: rendering_performance, build_reliability, reviewer.
Updated: 2026-10-14. Planning baseline: `f5637a1`.

## Outcome and boundaries

Compare Rust-Skia Ganesh over Metal with an explicitly pinned Vello GPU version/variant
inside a native macOS host. Deliver reproducible experiment code, exact dependency and
hardware identities, raw samples, visual comparisons, integration risks and a recommendation.
The production renderer remains OPEN until the owner accepts a decision.

This task creates an isolated experiment, not a production renderer abstraction. Preserve
the existing AppKit/CoreGraphics prototype, shared document format and production FFI.
Imported-image and clipping fixtures exercise renderer capabilities; they do not implement
new document features in the product. Do not present Vello CPU results as GPU comparisons.
No permanent backend integration, merge into main, release publication or paid equipment
purchase is included in this execution plan.

## 1. Freeze the experiment contract

Lead and rendering specialist agree the contract below before concurrent implementation.
The lead owns contract changes; version the manifest schema when it changes.

### Candidate versions

Initial researched candidates (superseded by the execution selection below):

| Component | Proposed pin / settings |
| --- | --- |
| Rust-Skia | `skia-safe =0.153.3`; Ganesh, Metal and binary cache; unrelated defaults disabled |
| Vello | `vello =0.8.0`; compute-centric GPU variant, `use_cpu=false` |
| GPU access | `wgpu =28.0.0`; Metal backend only |
| Apple interop, if needed | `objc2 =0.6.3`, framework crates `=0.3.2` |
| Native host | Minimal Swift/AppKit window and Metal surface; `winit =0.30.13` is an alternate host implementation only if needed |

These are proposed experiment pins, not an assertion that Vello 0.8 is the latest release.
Before coding, confirm published versions/features, compatible Rust/Swift requirements,
resolved Skia upstream revision, official binary availability and licenses. Explicitly
justify testing compute-centric Vello versus the newer sparse-strip `vello_gpu` family.
If the latter is the relevant maintained candidate for this product, revise the candidate
and lockfile before collecting measurements. Do not silently combine results across variants
or upgrades. A third candidate is a separately labeled experiment, not a substitution.

Commit a standalone experiment Cargo.lock and record compiler, SDK, build flags, Skia
source/binary identity and lockfile hash. Build release artifacts before timed work. Do not
raise the production workspace's minimum Rust version solely for this experiment.

### Execution selection (2026-10-14)

The approved preflight selects `skia-safe =0.153.3` Ganesh/Metal versus
`vello_gpu =0.3.0`, `vello_common =0.3.0`, and `wgpu =30.0.0` (Metal only).
The current official Vello guidance identifies sparse-strips Vello GPU as the intended
production GPU direction and places compute-centric Vello under research. The candidate
preprocesses paths on the CPU and rasterizes/composites on the GPU; include that CPU work
in measured stages. Results apply to these exact variants, not every Vello implementation.
The standalone lockfile will pin transitive dependencies. Production renderer selection
remains OPEN.

Use one Metal device/queue and equivalent offscreen BGRA8Unorm textures for both adapters,
then a common native drawable blit/presentation path. Common sentinel timing is labeled
GPU timeline span, including queue scheduling/idle gaps; it is not pure GPU execution.
Safe interoperation and actual timing availability must pass the smoke gate before data.

### Identical input and rendering intent

- Generate stroke outlines through the actual `sketch-core::Editor::render()` path.
  Group commands by stroke identity and nonzero-fill each complete outline once; separately
  alpha-blending its overlapping component shapes would change translucent appearance.
- Both adapters consume the same frozen command stream, transforms, clip paths, order,
  background, premultiplied-alpha convention, color space and decoded image bytes.
- Hash the generated geometry, fixture specification, input trace and image bytes. Assert
  that both runs use the same hashes. Record seeds and object/sample counts.
- Start at 1600x1000 physical pixels; add 2560x1600 after the common path passes validation.
  Record window points, backing scale and actual drawable size independently.
- Use equivalent source-over blending and image filtering. Specify each backend's actual
  antialiasing mode. Equal sample counts do not prove equal quality; quality validation is
  a gate before interpreting performance.
- Use the same invalidation policy. Separate full scene rebuild from retained static scene
  reuse; do not rebuild one backend while presenting a cached bitmap from the other.
  Backend-specific intermediate textures and necessary uploads remain part of measured work.

### Fixture matrix

| Fixture | Deterministic content and action | What it reveals |
| --- | --- | --- |
| Live stroke | 240 recorded pressure samples growing over 100 committed strokes; repeat a named trace | Incremental input/geometry/scene work and first-use stalls |
| Dense static | 100, 1000 and 5000 strokes, 24 samples each; fixed view | Encoding cost, retained-scene cost, geometry throughput and memory |
| Pan/zoom | Same 1000-stroke scene; fixed 600-frame transform trace | View updates, raster work and cache invalidation |
| Translucency | 256 overlapping outlines with fixed alpha values and draw order | Correct stroke unions, overdraw and blending |
| Clipping | Rectangular and curved clips with strokes crossing boundaries; bounded nested clips | Clip correctness and cost |
| Imported image | Identical decoded 2048x2048 RGBA image, vector overlay and scale trace | Upload, image orientation, filtering, compositing and retained resources |

Trace scheduling is frame-indexed for the primary comparison, independent of backend speed.
For live ink, frame `f` displays samples `0..(f % 240)` of one temporary stroke over exactly
100 committed strokes. Before a cycle restarts, cancel the temporary stroke; never append
committed strokes between cycles. For pan/zoom, use transform `f % 600` from the frozen trace.
Other animated fixture state is likewise a pure function of fixture ID and frame index.
Warm-up uses indices 0–119; reset logical fixture/viewport state, while retaining warmed GPU
resources, before measured indices 0–599. First-use capture is a separate reset phase.
Record a geometry/transform state hash per frame or hash the complete expected state sequence.
No wall-clock event backlog or skipped input changes the work assigned to a measured frame.
A real-time input-delivery/coalescing experiment, if later added, is separately named and is
not pooled into this matrix or described as physical pen latency.

The primary cache policy is retained vector scene/resources for immutable fixtures, with a
fresh rasterization each frame (no cached final bitmap). Live ink updates scene geometry each
frame; pan/zoom updates the transform using each backend's supported mechanism. Encode all
necessary changes and include them in CPU timing. First-use records full scene construction.
A forced-full-rebuild experiment is deferred, not an extra hidden matrix dimension. Document
any adapter that cannot implement this same invalidation policy before interpreting its cost.

These sizes are experiment workloads, not supported product limits. A documented stress
extension may be added after the primary matrix; do not tune the matrix after observing a
preferred winner. Image decoding is measured separately from steady rendering; first GPU
upload is explicitly included in first-use results.

Exit gate: written contract, chosen variant/version, deterministic fixtures, output schema
and measurement definitions agreed. No performance conclusion at this stage.

## 2. Prove native presentation and trustworthy timing

Use a small dedicated AppKit window with a fixed canvas and standard native controls for
backend, fixture, run/stop and result location. No UI redesign of the main application.
Automated runs set the same options through a CLI, so clicking speed cannot affect results.

Preferred pipeline: equivalent offscreen Metal output textures from both renderers followed
by a shared native final copy/composite and drawable presentation. Verify texture/device
ownership, format, color-space conversion, synchronization, queue ordering and lifetimes.
Keep necessary Vello intermediate/blit work and Skia flush work in the appropriate stages.
No CPU readback is allowed in the timed presentation path.

If safe shared-texture integration is impractical, use separate native presentation adapters
with identical drawable size, color format, display synchronization and in-flight policy.
Report their costs separately and disclose the presentation difference. Do not call the
pipelines identical or use a presentation-confounded result to declare a definitive winner.

Run a tiny smoke fixture before building the full benchmark:

1. Both backends render on the same physical Metal device; reject software/CPU fallback.
2. Readback outside timing verifies geometry, alpha, image orientation and color channels.
3. Native presentation remains correct during resize/minimize/restore and release of resources.
4. Timing queries resolve, missing/unsupported samples remain missing, and GPU completion
   cannot be mistaken for CPU submission.

### Timing definitions and feasibility gate

Measure CPU stages independently: input application, outline generation, backend scene
encoding, drawable acquisition, renderer encoding/flush/submission, completion wait and
presentation enqueue. Also capture wall-clock frame/service intervals without double-counting
nested stages. Use monotonic clocks and explicit units.

Vello offers GPU profiling/query support; document the exact included GPU passes and query
resolution. The proposed Rust-Skia safe FlushInfo API does not expose GPU statistics
callbacks. Investigate an isolated binding shim or a Metal command-queue timestamp bracket.
A bracket containing queue scheduling/idle gaps is a **GPU timeline span**, not pure GPU
execution time. Do not compare it directly with a sum of Vello pass execution times.

Prefer comparable GPU timing boundaries for both candidates. Record native internal pass
statistics as additional backend-specific diagnostics. If trustworthy comparable GPU timing
is unavailable, preserve the partial evidence, identify the limitation and keep the selection
provisional. Do not fill absent GPU timing with CPU elapsed time or zero.

Use two explicitly separate modes:

- **Completion diagnostic:** one frame in flight, wait for GPU completion. Useful for isolated
  work and readback validation; not representative of normal pipelined frame pacing.
- **Native paced presentation:** identical display synchronization and maximum in-flight
  policy (initial target: two). Measure acquisition waits and actual supported presentation
  events. A present call returning is not evidence that pixels reached the display.

Record the measured display refresh/pacing target. Report CPU/service budget overruns
separately from actual presentation deadline misses. If presentation timing cannot establish
missed display deadlines, mark that metric unsupported; do not infer it from present-call time.
Physical pen-to-screen latency requires suitable equipment and is otherwise **not measured**.
Synthetic input-to-submit measurements must retain that name.

Exit gate: verified native GPU smoke test, documented timing boundaries, real timestamp
availability and presentation behavior. Resolve measurement defects before collecting the matrix.

## 3. Implement the reproducible experiment

Proposed layout (no files implemented yet):

- `experiments/renderer-comparison/`: standalone Rust package/lockfile, shared fixtures,
  the two backend adapters and experiment-only FFI as required by the AppKit host.
- `experiments/renderer-comparison/macos/`: minimal native host and surface lifecycle.
- `scripts/build-renderer-comparison`, `scripts/check-renderer-comparison`,
  `scripts/run-renderer-comparison`: reproducible build, correctness and serial run entry points.
- `scripts/analyze-renderer-comparison`: deterministic summaries and plots from raw data.
- `docs/evidence/002-renderer-comparison.md`: tested revisions, results, risks and recommendation.
- Ignored `artifacts/renderer-comparison/<run-id>/`: raw samples, manifests, images and reports.

The CLI selects backend, fixture, dimensions, mode, warm-up/sample count, repeat ID and
output directory. Invalid options, failed initialization, missing required metrics and failed
validation must produce clear nonzero exits; partial runs must not appear complete.

Data contract:

| Artifact | Required contents |
| --- | --- |
| `manifest.json` | Schema version, source revision/dirty diff identity, dependency/compiler/SDK/build identities, OS/CPU/GPU/RAM, display/scale/refresh, power/thermal observations, backend/mode/quality, fixture/trace hashes, counts, timing provenance and unsupported fields |
| `frames.csv` | Run/backend/fixture/frame IDs, warm-up flag, each CPU stage, supported GPU timestamps/span, presentation information, deadline classification, memory samples and explicit missing reasons |
| `summary.json` | Counts, p50/p95/p99/max, misses/denominators, first-use stages, paired-run variation and sustained-window summaries |
| PNG/readback files | Matching fixture/frame IDs, output dimensions, reference/source hashes, visual comparison and difference views |
| Run log | Commands, failures, resource/thermal observations and interruptions |

Tests should exercise deterministic geometry identity, stroke grouping/alpha correctness,
input/manifest validation, missing timestamp handling, partial-run failure propagation and
summary statistics using small known datasets. Do not add tests that merely repeat settings.

Exit gate: clean-checkout build, both adapters passing correctness/failure checks, no unresolved
ownership or synchronization defects, and scripts capable of producing a complete smoke run.

## 4. Validate visual quality before performance

Capture matching final and selected intermediate frames outside timed runs. Inspect live
stroke joins/caps, translucent unions, clip edges, transform continuity and image orientation,
filtering/color/alpha. Generate side-by-side and difference images; report descriptive pixel
error measures without inventing an equivalence threshold after seeing the images.

The designer reviews native controls, visible rendering differences and run/stop behavior.
A visibly incorrect backend does not win by being faster. Repair and recapture; if a quality
setting cannot be matched, label the tradeoff and keep those results separate.

Exit gate: intended visual semantics match, remaining antialiasing differences are understood
and documented, and fixture settings are frozen for measurement.

## 5. Run serial measurements on the host

Prior measured host context is Apple M2 Pro /16 GiB; re-read and record actual hardware,
OS/SDK, display, AC/battery, low-power mode and thermal state before running. This host is
an available test machine, not an accepted universal reference device.

The lead owns the measurement window. Finish all agent builds first; pause task-controlled
emulators, development servers with animation and native automation. Do not terminate the
user's unrelated applications or tasks. Record unavoidable competing activity and repeat
contaminated measurements with the original data retained and labeled.

Primary-trial protocol for each of the eight workloads × two resolutions × two modes:

1. Start a fresh process and record initialization, first encoding, first upload and first
   complete/rendered/presented frame separately. Call these process-first-use results;
   filesystem/driver caches mean they are not necessarily cold-system results.
2. Warm up 120 frames, then record 600 frames. Preserve warm-up separately.
3. Repeat five paired trials, alternating Skia/Vello order across pairs. Use fresh processes
   and the same trace/start state. A longer diagnostic trial is allowed if necessary to
   characterize tails; record why and apply it symmetrically.
4. Capture thermal/power state before/after each trial and observed interruptions. Do not
   silently discard slower runs or change settings to rescue a preferred candidate.
5. Record process peak resident memory and available physical-footprint/GPU-allocation
   counters separately. Unified-memory figures may overlap; do not sum them into a false total.

Separately, run only the 1000-stroke pan/zoom workload at 1600x1000 in native paced mode
for 180 seconds after a 120-frame warm-up. Use the same frame-indexed 600-transform cycle,
advancing once for every submitted frame without accumulating input events. Record elapsed
time, submitted/completed frames and cycle coverage, including dropped/missed presentation
information. Repeat three paired trials across two backends: six sustained runs, 18 minutes
of measured wall time total. This is not repeated for every primary matrix cell. Sustained
live-ink accumulation, the higher resolution and completion mode are explicitly outside this
first sustained matrix; add them only as a documented follow-up if the initial results warrant it.

The initial matrix has eight workloads (three dense sizes plus five other scene classes),
two resolutions, two execution modes, five pairs and two backends: 320 process trials.
At a 60 Hz pacing target, the paced half alone contains about 32 minutes of warm-up/measured
frames; sustained runs add 18 minutes. Unpaced diagnostic runs, initialization, readbacks,
cooldown and repairs add variable time. This is a workload budget, not a delivery ETA.
First run a single low-resolution pilot pair; expand only after correctness, instrumentation
and data collection are validated. Any reduction in the declared matrix must be stated as
incomplete coverage, not silently counted as full task completion. Reserve an initial
measurement window of roughly 90–120 minutes after builds/pilot, with an explicit four-hour
contingency cap for that window. If startup/cooldown/repairs consume the cap, preserve the
completed cells and list pending ones for a subsequent window; do not silently weaken the
matrix, fabricate results or extend host occupancy without reporting the change. This is a
proposed execution budget for review, not a measured duration or product performance target.

Use an out-of-band memory sampler at a recorded interval plus OS peak counters; quantify
instrumentation overhead with profiling enabled/disabled on a small pilot. Do not insert
expensive per-frame memory queries and attribute their cost to the renderer. With 600 samples,
the per-run p99 has few tail observations; report its limited precision and extend both paired
candidates symmetrically if a decision depends on an uncertain tail difference.

Report per-run distributions and paired differences, not only an average pooled across all
frames. Include p50/p95/p99/max, deadline misses with denominator and definition, and variation
between runs. Treat differences within measurement noise as inconclusive. No universal FPS,
latency or memory acceptance target is invented by this experiment.

If repeatability is poor, diagnose contention/thermal behavior/instrumentation before expanding
scope. After two unproductive repair cycles, lead investigates or narrows the experiment
explicitly. Unsupported metrics/failed candidates remain visible in the report.

Exit gate: complete manifests/raw data for the declared matrix, interpretable measurement
boundaries, no unexplained correctness or timing failures, and documented unsupported cases.

## 6. Integrate, review and recommend

- Run `./scripts/check` for integration/scaffold changes and the experiment-specific checks.
  Identify tested commits and recheck affected behavior after integration.
- Build reliability specialist reproduces commands and validates data completeness and
  analysis; any GPU reproduction occurs serially in a separately reserved host window.
- Fresh reviewer audits the exact diff, unsafe ownership/FFI, fixture equivalence, query
  scope, frame/memory/deadline definitions, raw samples and interpretation. Resolve findings
  and rerun only the affected cases, retaining superseded results with their revision.
- Present a recommendation weighing correctness, live/tail responsiveness, first-use and
  sustained behavior, memory, integration complexity, build burden and future platform APIs.
  Report a tradeoff or insufficient evidence when appropriate; do not force a single winner.
- macOS results do not certify iOS, Android, Windows, Linux or browsers. Explicitly list
  follow-up backend/API validation required by the platform tasks.
- Draft an evidence-linked decision proposal. Mark the production decision accepted only
  after owner acceptance; implement production integration in a subsequent scoped task.
  Do not merge into main or publish automatically.

## Agent ownership and sequencing

| Phase | Owner | Scope / stopping condition |
| --- | --- | --- |
| Contract/preflight | Lead + rendering_performance | Freeze versions, inputs, presentation/timing definitions before parallel writes |
| Renderer implementation | rendering_performance | Isolated worktree; experiment crate/adapters/native host; stop at tested smoke/correctness handoff |
| Build/evidence tooling | build_reliability | Separate isolated worktree; named scripts/analysis/tests after contract agreement; no overlap in adapter files |
| Native UX/quality review | ui_ux_designer | Read/review the narrow host and comparison images; automation coordinated through lead |
| Benchmark execution | Lead or one explicitly assigned specialist | Exclusive host; no concurrent agent builds or UI automation |
| Independent review | Fresh reviewer | Exact integrated revision and evidence; report actionable findings and limits |
| Integration/report | Lead | Cherry-pick, verify, document and present owner review; main unchanged |

At most two implementation writers and three subagents concurrently. Every handoff names
starting commit, owned paths, contract version, dependencies, acceptance cases, evidence and
stop condition. Use isolated worktrees and serial host measurements; no recursive delegation.

## Completion checklist

- [ ] Candidate versions/variant and shared input/presentation contract frozen.
- [ ] Both GPU backends built and displayed in a native macOS host.
- [ ] Trustworthy timing boundaries established; unsupported metrics explicit.
- [ ] All six scene classes validated visually; quality differences documented.
- [ ] Raw first-use, repeated steady and sustained measurements retained.
- [ ] CPU/GPU timing, tails, deadline definitions and memory reported honestly.
- [ ] Reproduction/failure checks, required integration checks and fresh review complete.
- [ ] Recommendation and platform risks delivered for owner acceptance.
- [ ] Decision acceptance and production integration remain separate owner-controlled steps.

## Upstream references

- [Rust-Skia, including the native Metal example](https://github.com/rust-skia/rust-skia)
- [Rust-Skia GPU feature configuration](https://github.com/rust-skia/rust-skia/blob/master/skia-safe/README.md)
- [Vello renderer families and examples](https://github.com/linebender/vello)
- [Vello release history](https://github.com/linebender/vello/releases)

These are research inputs; the executed experiment must record pinned source identities
rather than depending on moving upstream default branches.
