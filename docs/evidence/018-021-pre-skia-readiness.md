# Tasks 018/021: macOS prerequisites for Skia

This is a scoped technical handoff, not all-platform acceptance or permission to merge.
The owner requested completion of the remaining prerequisites for task 019 and the task 020
Skia migration. The board retains its existing owner-acceptance and dependency rules.

## Audit and repaired gap

Two specialist audits and a fresh independent reviewer inspected the implementation at
`f0c9bc297b1d2426c7faf6272a7a031b5275f962` and its existing evidence.

| Task | Implemented scope | Work identified by this audit |
| --- | --- | --- |
| 018 | Cohesive Rust model/input/history/scene/codec modules; typed commands, session/interaction identity, immutable queries and capability enforcement; legacy ABI 2 and Wasm 1 retained | Add a current separately compiled legacy ABI 2 C consumer; the current macOS C/Swift consumers exercise native-v1, a different interface. Reconcile stale evidence text. |
| 021 | Bounded admission, immutable snapshots, serial storage worker, correlated acknowledgements, diagnostics, checkpoint/recovery services and task 019 native adoption | Refresh current execution evidence and remove obsolete claims that macOS service/UI adoption is still unimplemented. No new macOS implementation defect found in the scoped audit. |
| 019 | AppKit composition and shared Rust command dispatch, asynchronous file/recovery UI and native focus handling | Remaining resize/deactivation tests passed in attempt 32; retain exact revisions and prior designer review. No product changes or additional desktop tests are needed for this test-harness/documentation work. |

The new C consumer uses the authoritative legacy header and statically links the actual
Rust library. It checks frozen layouts, borrowed-input copying, invalid-batch atomicity,
prediction exclusion, frozen document bytes, output-capacity failure, cancellation,
invalid/imported document ownership, history, saved revisions and handle lifecycle.
Explicit checks remain active with `NDEBUG`. Deliberately wrong call expectations and a
corrupted golden fixture must fail before the passing conformance result is accepted.
The standard macOS background check runs this consumer; its separate evidence command
records actual executable discovery and execution, source/library/executable identities
and retained failure logs. No production API or behavior was changed.

Review reproduced a reporting defect in the initial consumer runner: a setup failure
published an enumeration scope before the executable had run. Such a scope would turn
missing discovery into an invalid-mapping report. The repaired runner publishes scope only
after successful actual enumeration; failed preparation retains no discovered tests or passes.
Worker commits `b3139bf` and `82a6f9f` are integrated as `468bf73` and `3f6ca94`.
Fresh independent review reran both negative controls, the passing consumer and a real
subprocess setup-failure regression and found no remaining actionable issues. Its artifacts
are `/private/tmp/legacy-abi-closure-review-82a6f9f/`. Final primary integration evidence
is added below after verification of the frozen candidate.

The first primary acceptance report also exposed obsolete case-wide XCTest mappings in
the task 019 manifest. Without a current desktop enumeration these produced 48 mapping
issues, although every requested core/storage/C case passed. The twelve single-macOS rows
now use the existing per-requirement coverage representation, preserving every test ID,
automation state and gap. Absent native enumeration remains `missing-discovery`, never a
pass; a test missing from an actually enumerated target remains `invalid-mapping`. No runner
rule or acceptance threshold changes. The initial nonzero report is retained at
`artifacts/verification/018-021-closure-8f1abf3/acceptance.log` and its immutable history.

## Baseline verification

At clean `f0c9bc2`, the lead ran:

- Full background `node scripts/check.ts`: passing tooling checks, Rust workspace tests,
  format/type/lint/source/dependency gates, current native-v1 C consumer, feature-enabled
  failure controls, ordinary Swift build and headless dispatcher/save/reopen integration.
- `node scripts/check.ts --platform web`: actual Web/Wasm/native document conformance,
  frozen protocol pairs, TypeScript/Vite build, real Wasm adapter checks and Pages asset/cache checks.

Logs are in `artifacts/verification/018-021-closure-f0c9bc2/`. The expected corrupt-input
failure message in the headless smoke journey is part of a passing failure-path assertion.
The Rust specialist independently ran 100 core/legacy-FFI/Wasm tests with no failures.
Every handwritten source in those three scoped crates was within 200 physical lines.
The reviewer independently ran 12 diagnostic, worker-lifecycle, process-kill and retention
tests. The ignored child-writer fixture is invoked by the passing process-kill parents;
it is not presented as a standalone passing case. Its log is `independent-services.log`.

Native evidence is separate: attempt 32 passed two selected gesture cases at `5933c5d`,
out of 45 discovered. Other save/recovery/focus journeys and designer review retain their
earlier exact revision and build identities in [desktop validation](019-native-desktop-validation.md).
No full current-revision native suite or fresh physical-input pass is claimed.

## Remaining boundaries and ownership

| Remaining item | Tracking / consequence |
| --- | --- |
| Windows/Linux ABI and storage/thread execution | Existing task 021 service requirements remain missing; host adoption/conformance is coordinated with 022 and the Windows/Linux port tasks. Mac results cannot close these rows. |
| Native provider coordination, ACL/xattr preservation and Windows replacement guarantees | Existing local-store contract preserves standard Unix mode bits, not universal provider metadata or power-loss guarantees. Native adapter/distribution validation remains required. |
| Snapshot capture and renderer scheduling | Snapshot capture remains synchronous O(document) outside pen callbacks. Task 020 must measure its effects and preserve requested-revision ownership; no bounded latency claim is made. |
| Device cancellation, physical pen, VoiceOver and broader accessibility | Existing 019/022/023 coverage gaps remain. The synthetic gesture passes do not certify these paths. |
| Production Skia, cache equivalence and surface/context recovery | Task 020 work, not a missing Rust extraction or storage implementation. |
| Full multi-platform refactor and source cap | Remaining host source debt and tools are tracked in 016/022/023/024. This milestone does not silently complete those tasks. |

The macOS/core slice can support task 020 after its technical checks and review. The
refactoring plan already permits authorized downstream local preparation while owner
acceptance is pending. Formal `done` statuses require acceptance of the stated scope and
dependency chain; the board validator is unchanged. Closing all of task 021 globally would
require the outstanding host evidence or an explicitly approved scope split retaining
those obligations. No task was silently narrowed, marked done, merged or published.
