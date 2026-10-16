# Task 022: portable adapter implementation evidence

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
`ReplaceForEachWithForLoop` violation in the extracted iOS scene entry. That repair is pending;
do not treat formatting as proof that every semantic lint rule passed. The fresh reviewer
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
records the source and log. The integrated source-size gate now reports three legacy
oversized files: Windows, Linux and web.

Portable provider facade/ownership, transport-v2 consumers, Apple/Windows/Linux/web extraction,
per-host Skia and real native journeys remain implementation/verification work. Windows and
Linux native acceptance require their actual hosts; physical device and camera checks remain
separate. The task board still reflects owner-acceptance dependencies, not these local handoffs.
