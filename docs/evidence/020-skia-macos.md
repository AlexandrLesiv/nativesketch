# Task 020: macOS Skia integration evidence

Implementation is on `codex/architecture-quality-plan`, descended from the owner's
`codex/github-pages` base. The source now uses Skia Ganesh/Metal with the existing AppKit
controls and shared Rust command dispatcher. Native checks and designer inspection ran;
final verification of the presentation-timestamp repair remains blocked by a system dialog.
The desktop reservation is authorized, but the computer-control tool refuses access to
that dialog. The owner has asked that no further per-app permission prompts be triggered
while absent. This is not task completion, owner
acceptance, a main merge or a release. Other platform migration gaps remain in
[rendering architecture](../architecture/rendering.md).

## Implemented boundaries

- Rust prepares immutable committed/live scenes, conservative viewport culling and
  reusable local-coordinate geometry. Unchanged document/viewport frames reuse the
  committed root. Predictions never enter committed caches or persistence.
- Skia owns ordered paths and a committed background; live ink is one fill and commits
  replace it atomically. Transform previews preserve original drawing order. A CPU
  reference bypasses Skia path/background caches while sharing the prepared scene.
- Swift owns the Metal surface, acquisition and presentation. The main thread owns both
  editor and GPU context; drawing is scheduled outside input callbacks. At most two
  frames are in flight with one coalesced pending repaint. There is no GPU completion
  wait. Drawable acquisition can still block and is measured, including failed attempts.
- Context reset clears disposable resources, waits for outstanding work to drain and
  bounds automatic recovery. Missing drawables receive two delayed retries, then wait
  for an external event. Document state remains separate from renderer failure.
- Presentation identity includes document/scene, surface, repaint generation, viewport,
  logical size/density, focus and semantic colors. Old callbacks cannot certify new pixels.
- Core retained geometry is bounded at 32 MiB, prepared geometry at 64 MiB, paths at
  32 MiB, background at 64 MiB and Skia resource cache at 64 MiB. These are separate
  budgets, not a total-process memory cap. Metadata/allocator/native surface storage are
  additional. Bounds metadata is capped by the document's 10,000-stroke limit.
- Large extents are clipped in f64 before narrowing to Skia floats. Exceptional cubic
  contours use bounded adaptive subdivision (1/1024 physical-pixel geometric tolerance,
  depth 32 and 262,144 points); resource exhaustion fails explicitly rather than drawing
  a partial scene. Reviewer-reproduced billion-coordinate endpoint loss is repaired.

Skia 0.153.3 uses standard default features plus Ganesh/Metal, following the owner's
October 16 direction. No custom minimal build is required. No authored C++ engine was
added; the C-compatible header is the Rust/Swift boundary. The former Core Graphics
painter remains unused reference code, without automatic production fallback.

## Background verification

| Tested revision | Check | Result |
| --- | --- | --- |
| `244fa4e` | Complete `node scripts/check.ts` | Passed: 71 Node tests, format/type/lint, source/dependency gates, Rust workspace tests, actual C consumers, Swift/Rust headless integration and six GPU-free Swift policy tests |
| `aaa83b4` | Native test app/bundle preparation | Build passed; execution intentionally blocked without desktop reservation; no test discovery or runtime pass claimed |
| `aaa83b4` | Release macOS application build/sign | Passed; no launch/GPU context |

The source-size gate retains five earlier legacy debts; new handwritten sources comply
with 200 physical lines. Two Rust subprocess-helper tests are marked ignored by libtest;
their parent process tests execute them. No skip was converted to a native pass.

Raw local evidence: [integration log](../../artifacts/evidence/020-background/integration-244fa4e.log),
[native preparation](../../artifacts/evidence/020-skia-prepared/identity.json),
[explicit preparation block](../../artifacts/evidence/020-skia-prepared/blocked.json),
[release build](../../artifacts/evidence/020-background/release-aaa83b4.log).
The preparation records exact app/test binary hashes. Later source changes require fresh
build identities; these rows certify only the revisions shown.

Core scene-cache coverage includes 11 integration cases; renderer coverage includes
11 CPU pixel/lifecycle cases. They cover ordered alpha, actual/predicted union, atomic
commit, transform placement, session replacement, eviction, resource failures, negative
and huge coordinates, zoom and backing density. Acceptance mappings are in
`quality/features/skia-macos.json`; mapping existence is separate from execution evidence.

## CPU cached/reference comparison

Measured isolated worker revision `7215616c91f9a91339a3d5d189381deeb0975284`, integrated
by `07845db`. Release binary SHA-256:
`57a473f11e7e7e68c5435bb26e9201491355d097d9664b7eacfefada397d3258`.
Apple M2 Pro, Mac14,10, 16 GiB RAM, macOS 27.0.1 build 26A434, AC power, no reported
thermal warning, Rust 1.99.0. Skia defaults + Ganesh/Metal linked; this experiment uses
CPU raster only and never creates a native window or GPU context.

Identical 1024×768 canvases, 128 strokes × 32 pressure samples, shared prepared scenes,
alternating cached/reference execution order and exact full RGBA equality after every
pair. All 200 pairs / 400 draws matched. Readback comparison is outside render timing.
Cold means renderer reset, not process startup. Warmup pairs are retained separately.

| Phase | Pairs | Cached p50 / p95 (ms) | Reference p50 / p95 (ms) |
| --- | ---: | ---: | ---: |
| Cold/reset | 10 | 5.885 / 7.724 | 5.789 / 7.445 |
| Warm unchanged scene | 32 | 0.276 / 0.294 | 5.545 / 5.931 |
| Live stroke | 32 | 0.338 / 0.452 | 5.783 / 7.654 |
| Ordered transform preview | 16 | 5.444 / 5.600 | 5.770 / 5.860 |
| Finite steady replay | 100 | 0.271 / 0.282 | 5.584 / 5.646 |

Caching substantially helps unchanged background and live ink in this CPU fixture;
cold frames and ordered transforms still redraw content. The four after-idle pairs and
single commit are retained in raw data but cannot establish robust tail distributions.
The roughly 2.71-second workload is not thermal endurance. Whole paired-process peak RSS
was 32,718,848 bytes and reported peak footprint 26,821,160 bytes; neither is a per-mode
comparison or a GPU-app memory measurement.

[Raw frames](../../artifacts/evidence/020-renderer-cpu-7215616/frames.csv),
[quantiles](../../artifacts/evidence/020-renderer-cpu-7215616/summary.json),
[host and binary identity](../../artifacts/evidence/020-renderer-cpu-7215616/host.json).
These results do not compare current Core Graphics with Skia Metal and do not measure
physical pen latency, input queues or display presentation.

## Native verification and retained attempts

Three new native cases check visible ink/undo/redo/minimize/restore, initial renderer recovery,
and recovery after a committed stroke with preserved visible ink and history. The injected
failure is at the Swift boundary, not a real Metal driver loss. Existing draw/save/reopen,
field shortcut priority, command routing, gesture cancellation and inspector/focus cases
are selected for regression. Final execution at the repaired revision is pending.

| Attempt / source | Observed result | Qualification |
| --- | --- | --- |
| native 1 / `9c424c5` | Seven raw cases passed; drawing pixels/recovery cases failed | Found real hosted-layer invalidation bug; fixed `f6765b3` by scheduling explicit repaints |
| native 2 / `f6765b3` | Draw/save/reopen and both recovery cases passed with runtime provenance | Hide shortcut case failed; the app has no Hide shortcut binding; batch did not pass |
| native 3 / `471548d` | Ink/history assertions passed; sandbox cross-app hide returned false | Replaced test mechanism with existing native minimize and AX restore; no permissions changed |
| native 4 / `e117245` | Minimize/restore case passed with runtime provenance | Strengthened baseline after confirmed minimization in `06b0c4d`; final rerun pending |
| policy / `8b3e3e3` | 14 GPU-free Swift tests passed | Timestamp classification, pending retry ownership, frame bounds, recovery and evidence preservation; not native execution |

The raw XCTest bundles, results, app/test hashes and provenance are retained in
`artifacts/evidence/020-skia-native-attempt1` through `attempt4`. The current batch runner
conservatively blocks otherwise passing rows if the whole XCTest invocation fails; raw
per-case passes above are not a claim that those batches passed.

Designer inspection of `e117245` found no blocking visual regression in visible ink,
selection, transform undo, pan/zoom, field priority/validation, small dark layout and resize.
See [designer report](../../artifacts/evidence/020-skia-designer/review.md) and screenshots
beside it. The reviewer explicitly did not use presentation counters as timing proof.
Live translucency/predictions and physical pen/VoiceOver speech remain uninspected.

## Release presentation audit

Release attempts 2–4 at `48dc2b5` used the Cintiq Pro 22, 120 Hz, 1000×699 logical /
2000×1398 physical canvas, 1000 eight-sample strokes plus 120 live steps. All three
original reports incorrectly say `valid=true`: their cold and after-idle presentation
timestamps are zero; attempts 2/3 also include a zero live timestamp. They are **invalid
complete presentation evidence**, not accepted latency results. Each retained directory
now has a `validity-audit.json` beside the untouched raw report and derived summary.
Do not use the negative derived latency values in those summaries.

Apple defines [zero presentation time](https://developer.apple.com/documentation/metal/mtldrawable/presentedtime)
as not presented or dropped. The cause of those compositor drops is not established.
`5f1a10f` rejects zero/nonfinite/pre-submission timestamps, preserves a qualified raw
observation and never certifies those frames. It adds at most two retries for the latest
request, preserving its initial request time. `8b3e3e3` repairs a reviewer-found stale-timer
race so a newer dropped request replaces the queued retry key. A successful later retry
does not erase a dropped frame or turn the strict benchmark into a pass.

Attempts 5/6 at `8b3e3e3` were rejected before drawing because an external system dialog
overlaps the canvas. No timing claim is made. Attempt 1 remains the earlier activation
failure, repaired by bounded activation readiness in `48dc2b5`. All attempts are retained
under `artifacts/evidence/020-skia-metal-attemptN`, with source and release binary hashes.
The production timestamp/retry repair has unit and independent static verification;
its final native behavior and release measurements remain pending.

The explicit `--render-benchmark OUTPUT.json --benchmark-revision REV` entry uses an
isolated runtime with persistent services disabled and synthetic shared commands. It
records cold, warm/static, live, commit, pan and after-idle phases, raw acquisition,
preparation/submission, queue/sample age, completion/presentation, cache sizes and failure
counts. Each phase requires its latest state to have presented. A 60 Hz visibility check
rejects hidden/covered/moved/resized/locked windows; validation overhead is reported.
This finite replay is not an endurance test or native pen-input test. Existing output
files must not be overwritten. Collect unique attempts and exact release binary hashes.

Fresh reviewer assessments found and prompted fixes for precision, selection cache reuse,
logical resize, missing acquisition timing, stale presentation identity, phase completion
and evidence preservation. Static designer review preserved native interaction intent;
actual designer inspection ran within the scope above. No full runtime accessibility, VoiceOver, physical
Pro Pen 3, camera latency, real GPU-device loss or untested display configuration is certified.
