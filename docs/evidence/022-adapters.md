# Task 022: portable adapter implementation evidence

Later portable-wire and browser E2E results are summarized in the
[background acceptance packet](024-background-acceptance.md). Earlier pending statements
below describe their recorded revision, not a superseding claim that later checks were absent.

This is an incremental technical record, not all-platform acceptance or a shipping claim.
The owner's instruction permits independent background work while the final macOS renderer
checks are blocked by an inaccessible system dialog. No further computer-control app permission
requests are permitted during the owner's absence. No merge, release or security change occurred.

## Shared actions and viewers

The [approved design](../architecture/022-portable-adapters.md) now has these core slices:

| Slice | Worker / integrated revision | Evidence |
| --- | --- | --- |
| Typed high-level actions and additive native-v1 action DTO | `968c096` / `41b183d` | Seven new core action tests, converter rejection and actual C compile/link/runtime; affected suites, rustfmt and strict Clippy passed |
| Issued readonly source replacement | `cf31372` / `1291f7c` | Four new viewer tests, full affected suites and actual C consumer passed |
| Registered executable mapping | `16af05c` | Real Cargo discovery/execution report: 236 acceptance/platform rows, 62 passed; remaining gaps are not passes |

Actions validate capabilities, committed selection, history, transform and capacity before
cancelling a draft. A valid scale action creates one undo operation and owns its anchor in
Rust. Primitive Undo still rejects active interaction; high-level Undo cancels then undoes.
The additive action DTO is 16 bytes; old primitive tags/layouts remain unchanged. Availability
is advisory committed-state availability; execution additionally validates context/geometry.

Readonly viewers receive private issued replacement tickets with increasing source generations.
Only the latest validated result can install; corrupt/cancelled/stale results retain the last
document. Installation does not create a save binding or grant edit/export/ordinary Open authority.
The local native-v1 filesystem runtime explicitly rejects the new viewer route; portable provider
adoption is separate work. No unimplemented host integration is inferred from core tests.

The fresh reviewer independently ran the seven action and four viewer tests, all passing,
and found no actionable correctness defect. Additional capacity/output-sentinel coverage was
recommended; no native UI was exercised by this review. Source-size compliance was checked.
[Discovery/results/report](../../artifacts/evidence/022-actions-16af05c/coverage.html) preserve
exact source and binary identity. The report's generated-success status is not full coverage.

## macOS shared action adoption

Worker `b26dc600` / integrated `0ccb071` routes tool selection, selection cycling,
deletion, undo/redo and one-step scaling through the shared high-level Rust dispatcher.
Native controls and menus use shared action availability; field editing retains native
first-responder priority. Local pointer capture clears only after accepted dispatch.

All 17 Swift policy/bridge tests passed, including three new `SessionActionTests`:
`unavailableUndoPreservesFirstDraft`,
`compoundUndoCancelsDraftAndUndoesCommittedHistory`, and
`invalidScalePreservesPreviewThenValidScaleCommitsOnce`. These use the real Swift/FFI/Rust
bridge without windows. Headless integration, Rust build, strict changed-file formatting
and native test bundle compilation also passed. Changed files are at most 175 lines.

Two new native held-pointer Undo journeys are mapped by `ACTION-NATIVE-HISTORY-001`.
Their execution remains pending; successful test-bundle compilation is not UI evidence.
Fresh review found empty-document selection cycling had become available; shared Rust
policy now rejects it before cancellation in `2d2a534`, preserving draft and predictions.
The reviewer independently passed that regression. The standard background gate now
includes the three bridge tests (`e5b35d6` / formatting `6e44ffb`). Full integration at
`2d2a534` passed, including all 17 Swift tests; the earlier formatting-only failure at
`e5b35d6` remains retained rather than relabeled. Native execution remains pending. Logs are retained under
`artifacts/evidence/022-macos-actions`; the worker revision identifies the tested source.

## Android mechanical decomposition

Worker `3e229db` / integrated `d88bafa` replaces the 1233-line Activity with native composition,
layout, presenter, file callbacks, pointer/input, painter, retention and replay collaborators.
All 26 handwritten Kotlin files are at most 186 physical lines after formatting. Existing
47 replay assertions and UI strings are retained; no new control or drawing feature was added.
The reviewer inspected native lifetime, receiver scope, field shortcuts and replay parity,
including a literal comparison, and found no concrete extraction regression.

Worker `398261d` / integrated `585bdf9` adds project-local ktfmt 0.64, formatting/source-size
checks and `androidQuality`. Clean committed dual-ABI ARM64/x86-64 APK build passed; APK SHA-256:
`4e2b9a95231bb495d71b76f926f70abc4dea173106a1fa10f5526f6d17996a1d` (9,429,968 bytes).
Android Lint reports zero errors and 15 retained unsuppressed warnings. JVM test task reports
`NO-SOURCE`, not executed test coverage. No emulator, native replay or Android/ChromeOS E2E ran.
[Identity and logs](../../artifacts/evidence/022-android-composition/identity.json) are retained.

This slice deliberately retains legacy JNI, Android Canvas and local persistence/navigation
policy. Those still need the portable Rust facade and Skia adapter; extracted host fields alone
are not the final architecture. Formatting is an installed gate; native feature acceptance is not.

## Apple compatibility decomposition

Worker `ce9ac082` / integrated `de54fe0` separates UIKit composition, lifecycle, native
controls, input, painting, file callbacks and replay. Shared viewer modules separate source
ordering/comparison, HTTP transport/cancellation and cache persistence. A dedicated legacy
module map in `apps/apple-bridge/CSketchFFI` repairs Apple builds without restoring legacy
imports in the macOS production adapter. Existing selectors, controls and behavior remain.

All 39 owned Swift files are at most 185 physical lines after formatting. Host viewer/source
replay passed; ARM64 simulator release builds for iOS, visionOS, tvOS and watchOS passed.
[Build identities and logs](../../artifacts/evidence/022-apple-extraction/identity.json)
record exact binaries. No simulator was launched, and no device/UI/Pencil/paired-Watch check
ran. Portable facade adoption and Skia rendering are separate remaining changes.

Formatter checks passed, but the integrated broad Swift lint at `de54fe0` found one
`ReplaceForEachWithForLoop` violation in the extracted iOS scene entry. `0c08954` repaired it;
`487a5de` added strict lint to the integration gate, including manifests and test sources.
All 133 files passed with explicit selected Xcode at `735754d`. The fresh reviewer
found no extraction defect in ownership, selector/delegate identity, source ordering or
module-map resolution; the quoted Swift literal set was unchanged. The reviewer independently
reran the host replay binary with the recorded hash and it passed. This does not add native
simulator/device acceptance.

## Baseline and outstanding hosts

At `24cf21a`, the web build, actual Wasm adapter checks and production precache/subpath check
passed. These are build/replay checks, not browser E2E. Apple viewer replay initially failed
before execution because its script referenced the removed macOS `CSketchFFI` module; the
Apple extraction above repaired that failure. Both baseline logs are retained under
`artifacts/evidence/022-baseline`; the failure is not hidden by the passing macOS integration gate.

The existing GTK adapter also passed a background `cargo check` against installed GTK 4.24.1
and libadwaita 1.10.0 on macOS. This checks compilation only: no Linux, Wayland, portal or
native accessibility behavior was exercised. `artifacts/evidence/022-linux-host-probe`
records the source and log. Subsequent composition slices are recorded below.

Portable provider facade/ownership, transport-v2 consumers,
per-host Skia and real native journeys remain implementation/verification work. Windows and
Linux native acceptance require their actual hosts; physical device and camera checks remain
separate. The task board still reflects owner-acceptance dependencies, not these local handoffs.

## Windows and Linux composition

Windows worker `0e4cd7c` / integration `d189475` separates composition, native controls,
presenter, input/viewport, painter and file effects into 13 C# files, maximum 192 lines.
Real WinUI compilation and C#/Rust interop passed on macOS. Source formatting passed;
design-time analysis reported the expected unsupported Windows resource tool warning.
This is not Windows packaging or runtime acceptance. The reviewer found no extraction
defect and independently reran the existing interop binary; a fresh reviewer rebuild was
sandbox-blocked, so that check is explicitly not a fresh-build claim. Logs are retained in
`artifacts/evidence/022-windows-composition`. Windows UI/DPI/pen/Narrator remain pending.

Linux worker `c7c5ded` / integration `15daa23` separates composition, commands, controls,
input, viewport, painter and file effects into 12 Rust modules, maximum 157 lines.
All 60 string literals and eight replay assertions remain. Format, strict Clippy, the
pure adapter test and GTK build passed against GTK 4.24.1/libadwaita 1.10.0 on macOS.
The initial missing-trait-import build failure is retained. The fresh reviewer found no
extraction defect and independently reran the pure test binary without initializing GTK.
[Identity and logs](../../artifacts/evidence/022-linux-composition/identity.json) identify
the exact source/binary. Linux display servers, portals, accessibility and pen remain pending.
Both hosts still use their legacy engine/storage/rendering adapters; composition is not v2
adoption or Skia migration.

## Browser composition and emitted asset graph

Worker `9e9baf9` / integration `5c05a7e` separates 15 source modules, maximum 154 formatted
lines, preserving all 120 original runtime string literals. It retains protocol 1, Engine
ownership, focus and pointer behavior, and honest download/dirty semantics. The production
precache now includes the complete emitted module graph rather than only main and engine.

Strict TypeScript 7, type-aware Oxlint, Oxfmt, actual Wasm adapter/native-byte conformance,
20 protocol fixture pairs and four asset/cache tests passed. Cache tests cover nested
dependencies, missing asset rejection, digest changes and failed-install retention of the
old cache. These execute the emitted worker in a Node fixture, not a browser lifecycle.
[Build identity and every emitted asset hash](../../artifacts/evidence/022-web-composition/identity.json)
also disclose the worker's untracked local dependency symlink and retained development
failures. Real browser E2E, CanvasKit and protocol-v2 adoption remain pending. Independent
review found one obsolete build-path regression, described below; no other introduced
correctness finding. The reviewer independently passed all four cache tests and the real
Wasm adapter checks, and verified all 21 emitted asset hashes against the worker identity.

The primary `node scripts/quality/check.ts --final` passed on source `5c05a7e`
(documentation-only evidence updates present): zero handwritten source files exceed
200 physical lines, and production dependency/native-boundary gates passed. This closes
the size-inventory debt, not provider migration, native testing or task 024 acceptance.

The retained shell builder did not substitute the new precache asset template, producing
a service worker that failed installation. The supported Node builder passed primary
web checks at `36fa8c0`, followed by actual loopback GET asset delivery,
Wasm/JS/HTML MIME checks and POST rejection. The first server attempt was sandbox-blocked;
the authorized background retry passed without opening a browser. The obsolete five
web/Wasm shell entry points and nested TypeScript/Prettier manifests were then removed,
with active docs routed to Node. Historical command evidence remains historical.

Root type-aware lint and formatting now include the extracted browser source and asset
tooling. Node and browser lint projects run separately with concrete file arguments;
an attempted literal glob caused a type-engine diagnostic and was repaired by enumerating
tooling files. The full core integration passed after that repair. Failed attempts and
server/build logs are retained in `artifacts/evidence/022-web-retirement`. Real browser
offline lifecycle acceptance is still pending.
Fresh independent review of cleanup `8e3dd13` confirmed the obsolete-path defect is
resolved, active callers are updated, both type-aware lint projects and TypeScript pass,
and all four cache tests pass. It found no additional actionable issue in that repair.

## Typed portable application facade

Worker `42bd477` / integration `e61a2bd` adds `sketch-app`, depending only on `sketch-core`.
It owns bounded sessions, resource leases and issued worker/result transport; core retains
all editing, navigation and save policy. It starts no thread and performs no I/O. Its
mutation owner cannot move between threads; uniquely owned work/results can. The
[public contract](../../crates/sketch-app/README.md) documents lifetime, cancellation,
budget, release and explicitly unconfirmed export behavior.

Fresh independent review reproduced and repaired two defects before freeze: redundant
same-file alias grants exhausted capacity, and retiring a stale load could leave its
navigation continuation busy. Exact binding-lease ownership and matching-continuation
cleanup now have recovery regressions. The reviewer independently passed all 26 facade
tests, strict Clippy and formatting at the frozen commit. Test binary SHA-256:
`ea32569dc4b9850869391b521476a8b3015cd19cf9950c9eb234ddc12c0c477f`.
All new Rust source files are at most 171 lines. The author's full background integration
also passed. C/JNI/Wasm transport, host adoption and native acceptance are separate work;
no portable recovery provider or platform renderer was introduced by this crate.

At primary `d4afbc7`, actual Cargo discovery/execution passed all nine registered facade
acceptance groups. The complete report has 246 acceptance/platform rows, 71 passed;
the remainder remain explicit gaps. [Revision-bound coverage](../../artifacts/evidence/022-facade-d4afbc7/coverage.html)
does not equate successful report generation with complete product coverage.
The full background integration at `3d97af4` passed, including all 17 GPU-free Swift tests.
Its first attempt passed portable checks but was sandbox-blocked reading native host
identity; the authorized background retry and original failure are both retained.

## Returned write receipt recovery

`5fe559c` adds explicitly unconfirmed export dispatch and checked encoded size. Unconfirmed
downloads never acknowledge durable saving or clear dirty state. Review found a rejected
write outcome consumed the only receipt while retaining the coordinator's active barrier.
`39453a0` returns an owned rejected receipt for correction/retry without releasing ordering.

Further review found native poison cleanup could lose that returned receipt and consume
a queued successor before it was runnable. `12557b8` settles an actual returned invalid
outcome as uncertain before poisoning, preserves blocked queued work, and includes outstanding
coordinator jobs in shutdown readiness. It also repairs the diagnostic counter bound for the
new export code. Unknown or mismatched receipts remain quarantined, never falsely completed.

At exact primary revision `12557b8`, the affected Rust/native suite passed 182 tests with
zero failures. Two ignored subprocess fixtures are exercised by their passing parent tests.
The new real-worker fault case verifies poison, detached cleanup, exact successor bytes,
single grant release and shutdown readiness. Fresh independent review cleared the repair;
the counter-bound failure and retries remain in `artifacts/evidence/022-receipt-recovery`.
The full background integration gate at the same revision passed, including 17 GPU-free
Swift bridge/policy tests. No new native UI or GPU presentation evidence is implied.
