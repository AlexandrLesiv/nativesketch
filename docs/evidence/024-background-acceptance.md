# Task 024: available background acceptance packet

Snapshot of executed evidence on 2026-10-16, not final all-platform acceptance. The owner
has prohibited computer-control actions that might request app permissions during their
absence. Native UI, GPU and simulator checks remain pending; background work continues.
No merge, publication, security-setting change or inferred owner acceptance occurred.

## Architecture and integration

The integration branch descends from `codex/github-pages@712059d5`, not the renderer-results
branch. Mac AppKit controls retain the shared Rust command dispatcher and the selected
standard Skia renderer. Other hosts have been decomposed; portable transport and provider
adoption are incremental, so extraction alone is not complete architectural migration.

| Evidence | Exact source | Result and limit |
| --- | --- | --- |
| Full background integration | `df2647f` | Passed: Rust checks, 98 Node tests, strict lint over 133 Swift files, pre-AppKit smoke and 17 GPU-free Swift tests; no native UI/GPU acceptance |
| Portable wire core integration | `e8d191f` | Passed before subsequent lifecycle/index repairs; final worker freeze separately tested below |
| Wire/facade acceptance | Worker `6df14a2` | Eight mapped groups passed, 32 wire tests and 27 facade tests; strict Clippy and core integration passed |
| Source inventory | `cfc9e9f` | 704 handwritten source files, 53,196 physical lines, zero files above 200 lines; this is a point-in-time inventory |
| Browser application E2E | `0967656` | Seven actual headless Chromium journeys passed against production Wasm assets; Canvas2D baseline, not CanvasKit or physical pen |
| Instrumented sanitizer | `df2647f` | Six legacy FFI tests passed; actual ASan symbols and deliberate fault detection verified; narrow scope below |
| Portable facade Miri | `cfc9e9f` | All 27 library tests passed under Miri; not native transport, host or GPU coverage |
| Installed facade Miri lane | `a3c7497` | All 27 library tests passed through the repeatable `miri-app` runner; fresh runner review passed |
| Linux portable owner/provider | Worker `342cfae`, integrated `a3c7497` | 15 tests passed plus one ignored child helper exercised by its parent; strict Clippy/format/build passed; native host and save-chooser compatibility remain pending |
| Bounded core fuzz | `f8bb00b` | Both targets generated 25,000 inputs and then failed LeakSanitizer in the isolated runtime thread path; no passing fuzz acceptance |

The final wire code was independently reviewed at `94d1503`; `6df14a2` adds executable
mapping only. Review found and repaired lost completion classification during save/load
cancellation and repeated linear geometry lookup. Direct indexed lookup and lifecycle
regressions are included. Native/Wasm exports and actual host adoption remain distinct work.

Raw primary artifacts are under `artifacts/evidence/`: `integration-df2647f.log`,
`integration-e8d191f.log`, `024-source-inventory-cfc9e9f.json`, `017-browser-0967656`,
`023-asan-df2647f` and `023-miri-facade-cfc9e9f`. Worker wire logs are retained at
`/private/tmp/wire-root-core-6df14a2.log` and `/private/tmp/wire-evidence-6df14a2.log`;
its checkout has the revision-bound coverage/discovery/results JSON and HTML.

## Browser and evidence integrity

The browser journeys exercise history/selection/scale/delete, native numeric-field focus,
Escape/resize cancellation, file download/reopen/corrupt retention, stale navigation choice,
and offline caching/update failure. A failed first resize attempt is retained; the test was
repaired to wait for the actual backing-store resize before releasing the pointer.

Fresh review found that a process timeout after writing passing test JSON could be
misreported as success. `0967656` makes the child process outcome authoritative for
infrastructure failure, preserves ordinary mixed assertion results, and records exit,
signal and failure reason. Nine runner tests include timeout/cancellation and output failure.
Normal quality checks now include the runner and browser E2E sources. Fresh review passed.
Actual Chromium at the primary revision passed all seven journeys with exit zero.

## Safety tooling and invalidated evidence

`023-asan-25266d5` is invalid sanitizer evidence: empty encoded Rust flags shadowed the
instrumentation flag. Its raw passing test result remains preserved and explicitly excluded.
The repaired runner verifies the actual test executable's instrumentation and runs a
disposable deliberate use-after-free canary. Canary failure is expected evidence; ordinary
crashes or absent instrumentation do not qualify. A later cancellation repair prevents
proof commands from continuing after the lane is cancelled; fresh review cleared it.

At `df2647f`, six instrumented legacy Rust FFI tests and the required canary passed their
respective checks. Test binary SHA-256:
`432945a5a16936bee3425fc067213286c28fd858b6313da64a10953b5ead3edf`.
Symbolizer warnings mean detection is proven, not fully symbolized diagnostic quality.
This lane does not cover Skia, JNI, Wasm, native UI or the portable facade.

The separate facade Miri run at `cfc9e9f` used pinned nightly `2026-10-15`, default isolation,
and passed all 27 tests. Its identity, output, result and subsequent tool-hash supplement
are retained. The later installed `miri-app` lane passed those 27 tests at `a3c7497` in
`artifacts/evidence/023-miri-app-a3c7497`; the original `miri` lane still targets the six
legacy FFI tests. [Bounded fuzz evidence](023-bounded-fuzz.md) retains failed executions and
an empty-target runtime reproduction. Fuzz acceptance and per-host profiling remain incomplete.

The [Linux adapter evidence](022-linux-portable.md) records real portable owner/provider
tests and repaired bounded, nonblocking diagnostics. Subsequent compatibility inspection
found that guarded Save must keep opening a chooser after earlier saves, as the original
GTK interface did. The current shared default may reuse the bound destination. A shared
chooser-on-save policy and regression are required; this slice is not final Linux acceptance.

## Package and symbol evidence

Release builds now retain Rust line tables (`55f723b`) without changing Skia features or
LTO. At `414d58a`, the saved macOS app and dSYM have matching UUID
`AA2EF5D1-0309-321C-BD2F-C3A1689D727A`; 64 owned Rust compilation units and Swift debug
information are present. Earlier `67083a3` symbol evidence covered Swift but lacked owned
Rust line tables; its separate scope correction remains with the original artifact.

The saved `414d58a` binary is 6,051,088 bytes, SHA-256
`459ec29e2ad983b98c0308d5499691eacf3f6e9716d0dacc7ab6997f9ea226df`.
Ad-hoc signature verification, system-library linkage, absence of test-control symbols and
pre-AppKit command/save/reopen smoke passed. Full app/dSYM copies and logs are retained in
`artifacts/evidence/024-package-symbols-414d58a`. This is not Developer ID signing,
notarization, clean-user installation, upgrade, distribution or native UI acceptance.

## Cross-platform renderer feasibility and remaining gaps

Standard Skia 0.153.3 release library compilation passed for iOS device, iOS simulator and
Android ARM64. Skia's Metal feature additionally compiled for iOS device/simulator. The
project Metal adapter was still macOS-gated: these results do not prove an Apple app link,
GPU draw or touch/Pencil behavior. visionOS lacked the requested prebuilt artifact; the
guarded source-build attempt is retained as blocked. tvOS/watchOS viewer renderer policy
requires a separate explicit adapter decision. Details are in
`artifacts/evidence/020-portable-skia-probe/REPORT.txt` and associated identities.

Outstanding acceptance includes current macOS native regressions and valid presentation
benchmarks, actual Windows/Linux/Android/Apple host E2E, portable recovery providers, remaining
Skia host adoption, packaging/upgrade/distribution targets, speech accessibility, physical
pen-to-screen camera measurements and retired-automation parity on all supported hosts.
The user's later developer-workbench work may proceed after this available local packet,
with these gaps visible. Task 024 remains open and is not owner-accepted.
