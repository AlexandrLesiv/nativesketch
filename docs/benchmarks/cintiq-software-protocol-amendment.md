# Cintiq collection amendment: preserve bounded performance failures

Date: 2026-10-15. Native source remains
`8a26d1d7a25bfcbc526584d386a4f49c30ecbbcf`; native binary, input, painter, cache,
canvas, ordering, durations and watchdogs are unchanged.
This amends the collector's halt policy in the
[software execution contract](cintiq-software-execution.md), before continuing acquisition.

## Observations motivating the amendment

The first session accepted four complete pairs (eight runs), then stopped because
macOS Dock window 172, layer 20, temporarily covered the Cintiq canvas. The failed
attempt and original collection remain preserved. A second recorded session reused
only the eight checksum-verified complete runs and restarted the affected pair.
The overlap guard was not changed. That environmental interruption is distinct from
performance failure and does not enter accepted timing distributions.

The second session reached fourteen accepted runs. Its first 180-second Core Graphics
cached long-stroke trial at 2560 × 1600 failed the existing terminal-draw watchdog.
All 43,201 samples reached the editor, but the last drawing callback represented
39,424 samples at 164.339 seconds. The next delivery resumed at 198.792 seconds; the
watchdog then fired at 198.931 seconds. The final 3,777 samples never reached a draw.
The last drawing interval was 24.764 seconds. A 76.57 ms latest-sample age obscures
this backlog: the oldest newly drawn sample was 24,826.57 ms old, and the oldest sample
in the terminal delivery arrived 34,525.27 ms late.

The **long trace is adversarial repeated retracing**, not a representative smooth,
spatially growing line. Its horizontal phase resets every two seconds, so a 180-second
stroke accumulates ninety overlapping passes and abrupt end-to-start segments in one
outline. This limitation applies to both renderers. Do not substitute an easier trace
after observing the failure or generalize its result to ordinary handwriting.
A more natural trace can be a separate, explicitly exploratory follow-up.

A separate 90-second diagnostic run, excluded from comparative timing, completed at
99.436 seconds. Three-second stack samples at approximately 50 and 80 seconds located
main-thread work in Core Animation backing-store updates and Core Graphics display-list
rasterization (`ripc_DrawPath`, `RIPRenderCoverage`, antialiasing edge distribution).
At the second sample, all 1,501 sampled main-thread stacks passed through that drawing
path. These sampled intervals support deferred rasterization as the slowdown's location;
they are not measurements of physical latency or the whole run's time distribution.

The watchdog runs on the main queue and cannot interrupt synchronous framework work.
Its grace condition is checked when the queue resumes, not enforced as a hard wall-clock
execution bound. A short `NSView.draw` callback excludes substantial deferred work;
process CPU, per-sample delivery backlog and drawing intervals must also be reported.

## Narrow continuation policy

Keep every native measurement unchanged. Freeze collector revision, runner/analyzer
hashes, native revision/build/binary identity and this explicit opt-in policy separately.
Verify that native source files have not changed between the native and collector
revisions; never relabel an old binary as a newer build.

Allow collection to continue only after the exact existing terminal-watchdog failure
in the declared 180-second, dense1000, long-trace, C-policy, warm condition, symmetrically
for either renderer and either declared size. Classification requires verified native
identity, configuration/display metadata, empty physical-input and lifecycle logs,
complete expected delivery sequence, matching expected final document/geometry hashes,
and a missing terminal drawing representation. The elapsed time must meet the existing
watchdog condition. Skia must also have completed GPU retirement.

Write a distinct checksummed failed-observation receipt. On resume, revalidate the
classification and every preserved artifact. Do not retry this outcome until it passes.
Stop, an occluding window, changed display, corrupt evidence, unexpected failure or a
failure outside this narrow condition still halts collection for diagnosis.

Report attempted, accepted, classified performance failures, other invalid attempts and
pending runs separately. A failed completion remains a failure, never accepted timing,
zero latency or missing data silently omitted from the recommendation. Show paired
completion outcomes even where no numeric timing difference can be calculated. Partial
failure timelines are descriptive and censored; do not pool them into successful-run
percentiles. Preserve all session boundaries and the original environmental rejection.

## Review and remaining scope

A fresh reviewer accepted the methodology and reviewed the integrated classifier and
paired-outcome repair at `8bb8560`. All 49 portable tests passed independently.
The declared Skia partner completed all 43,201 samples at 180.424 seconds; its
scheduled-to-first-draw p95 was 300.20 ms (maximum 439.33 ms). Input/display identities
match the failed Core Graphics partner. Retain its separate session boundary. This
one pair is descriptive evidence, not a final comparison or physical latency result.
No native threshold or quality requirement is relaxed. Physical pen-to-screen evidence,
real Wacom input characterization and the previously stopped bundle-size checks remain
pending. The permanent renderer decision remains open.
