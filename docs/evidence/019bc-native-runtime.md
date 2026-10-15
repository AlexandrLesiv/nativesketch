# Native runtime and macOS adapter migration — in progress

The earlier integration checkpoints below retain their original limitations. Subsequent
native batches passed 20/20 at `1672c49`, 1/1 and 15/15 separately at `79f3028`, and 13/13
selected regressions at `03cad1c`. See the current desktop validation report for exact
identities, designer review and remaining limitations; these are not one all-case final run.
This document does not certify the remaining refactoring plan or other platforms.
The branch remains a descendant of `codex/github-pages`; no main merge or release occurred.
Subsequent reserved-desktop execution, newly found native failures and repairs are recorded
in [desktop validation](019-native-desktop-validation.md); the earlier checkpoints below
retain their original scope.

## Integrated checkpoints

| Revision | Result |
| --- | --- |
| `c0b63da` / worker `106abfe` | Rust/Swift owners agreed native v1 declarations; actual C and Swift import probes passed |
| `4b9a37c` / worker `627ef7d` | Swift command adapter, projection presenters, native effects and lifecycle; typechecked, initially unlinked |
| `070e92a` | Production bridge import/header guard; seven focused tests, actual positive CLI, types/lint/format passed; independent review found no actionable issue |
| `3047916` / worker `861a28c` | First linkable Rust runtime: full integration run passed 59 Node tests, 133 Rust tests, and macOS compilation/link; headless Save As failed |
| `0f23fb5` / worker `5bfd3dd` | Worker resolves trusted macOS parent aliases before validating destination; final symlink entry still rejected; regression added |
| `b7528c2` / worker `7e7e589` | Native failure/presentation repairs and labeled smoke errors; integrated build and headless journey pass |
| `32095bc` | Recovery identity/allocation repairs and 20 source-authored XCTest cases integrated; full background check and actual Wasm compatibility pass |
| `f451c26` | Correlated worker diagnostics, authoritative Save As retry and caught-panic cleanup; full background check passes: 59 Node, 139 workspace Rust plus one explicit feature test; one existing child fixture ignored |
| `4c42862` / worker `c82f027` | Separate recovery authorization, correlated failure kinds, checkpoint scheduling, process-lock and stale-read checks |
| `21327bd` / worker `95f9da8` | Recovery completion matches exact session and identity; independent runtime review closes reported findings |
| `350017d` / worker `df804bd` | Stronger native test assertions for acknowledged dirty state and the exact second document's recovery revision; 20-case suite compiles, headless candidate passes |

The `f451c26` check also executes the real linked C consumer, verifies the fault-injection
symbol is absent from the normal release library and present only in the explicit feature
build, and passes the linked Swift headless journey. These tests run without native windows.
Environment: Mac14,10 arm64, macOS 27.0.1, Xcode 27.0, Swift 6.4, Rust 1.99.0, Node 24.22.0.
Logs are `check-f451c26.log` and `wasm-32095bc.log` in the integration verification directory.
The Wasm run executes real Node WebAssembly and 20 fixed protocol-v1 request/response pairs.
The coverage report at `32095bc` has 32 passing rows out of 204; this is explicitly incomplete
coverage, not an overall acceptance pass.

Final integrated implementation code `350017d` passes the full background check: 59 Node
tests, 144 workspace Rust tests plus one explicit feature-control test, actual C consumer,
normal/feature symbol separation, Swift build/link and headless integration. Two ignored
subprocess entrypoints are exercised by passing parent tests. Documentation-only evidence
updates occurred during this run; implementation/test sources were unchanged. Log:
`artifacts/verification/019-integrated-runtime/check-350017d.log`.
The Rust owner's all-feature lane separately passes 146 tests at `95f9da8` (same integrated
runtime code), including feature-only save-failure coverage. See
[runtime verification](../architecture/019bc-verification.md) for lane distinctions.

The one ignored Rust test at `3047916` is the existing child-process fixture exercised by
its parent kill/recovery tests, not a newly skipped acceptance case. The full run's headless
failure remains recorded: an ordinary temporary-directory path through `/var` reached
WRITE_ACCESS because its canonical parent uses `/private/var`. The resolver was repaired;
the fixture was not canonicalized to hide the issue.

The passing `b7528c2` headless journey uses the actual linked Swift/C/Rust runtime and local
worker for mouse/pressure geometry, stroke cancellation, transform history, asynchronous
Save As/reopen, corrupt input preservation and shutdown. It calls semantic commands directly;
it is not native keyboard/pointer E2E or physical-pen evidence. Its expected corrupt-file
failure is printed separately. Build/timeout logs are under
`artifacts/verification/019-integrated-runtime/`.

## Independent review and repairs still required

- Contract review caught unresolved-location FIFO ordering and permanently stale event
  disposal gaps before implementation. The contract rejects later I/O admissions while
  a resolution barrier is pending and defines terminal abandonment of stale candidates.
- Early worker review reproduced empty recovery registrations exhausting capacity. Lazy
  checkpoint allocation and verified-empty cleanup are implemented; broader closure remains.
- Swift review at `4b9a37c` found poisoned-close handling, uncertain/durability feedback,
  failures lost behind another sheet, obsolete progress, and rejected chooser continuations.
  Four scoped findings are closed at `b7528c2`. Shutdown admission ordering and subsequent
  stale-state/retained-event scheduling findings are closed by independent review at
  `f451c26`. Actual C/service probes reproduced poisoned idle leaving its output unchanged,
  and a stopped worker with one issued notification still preventing runtime free. The
  repair queries fresh state and drains retained notifications before stopping the pump.
- Runtime review at `3047916` reproduced cancelled recovery metadata resolving an unrelated
  later document's candidate, and first-allocation failure making later retry ineffective.
  It also identified missing read/recovery failure diagnostics and lost correlation.
  Repairs and regression tests are integrated in `0c486d9` and `f451c26`. Further review
  caught Retry authorizing a rejected recovery identity, then a delayed Retry updating a
  replacement session. Explicit approval and exact session/identity completion matching
  repair both success and failure paths. Independent review at `21327bd` closes the reported
  findings: eight independent probes, eleven service tests and one FFI panic test passed.
  A disposable combined-probe run had colliding fixture names; isolated reruns passed, and
  that fixture failure is retained in the review record. No finding was waived.
- A separate review of the 15 new native cases identified missing dirty-state acknowledgement
  and a timing-dependent recovery checkpoint assertion. `350017d` strengthens both; actual
  UI execution still remains pending and cannot be replaced by test-source review.
  Independent review closes both at that revision and independently compiles the native
  test target. No native journey pass is inferred from this compilation.

Review records and disposable probe copies are under
`artifacts/verification/019bc-contract/` and `artifacts/verification/019de-independent/`.
The runtime reviewer retained its independent reproduction crate under
`/private/tmp/native-runtime-review-3047916`.

## Remaining acceptance

A read-only macOS owner audit at `7ecfd01` found no production editing/persistence bypass:
tools, brush, selection/transform/history, drawing and file intents enter the typed session
dispatcher; recovery management enters the Rust runtime service boundary. This is an owner
audit, not independent review or native event-delivery evidence. Native-only field editing,
viewport mapping and presentation remain intentional. Compound native command sequences and
future agent equivalence are explicitly documented in the session bridge contract.

Deterministic worker controls, automated C-consumer execution, caught-boundary-panic cleanup,
cross-process locks, stale-read/grant retirement and Wasm compatibility have lower-level
evidence. The scoped runtime and shutdown review findings are closed; this is not exhaustive
FFI acceptance. Nine runtime acceptance rows map to discovered host executables in
`quality/features/native-runtime.json`; feature-only tests and the C consumer retain separate
raw evidence until dedicated discovery lanes are modeled. The 20 compiled native cases preserve the
five existing journeys and add asynchronous save, navigation, failure and recovery coverage.
Preparation and headless checks do not constitute executable XCTest discovery or execution.
The subsequent attempt 19 passed all 20 native cases at `1672c49`; the earlier timeout was
late system authentication. Expanded attempt 20 enumerated 36 cases at `79f3028` and passed
only the selected field-focus case. Its full background check passed 151 workspace Rust
and 59 tooling tests. Additional acceptance and designer-found repairs remain in verification;
see desktop validation for exact revisions and limits. Broader inspection, accessibility,
physical pen latency and other platform hosts remain separate, explicit evidence gaps.

All new/extracted source remains subject to the 200-line gate. Five other-platform source
debts remain. Production Skia integration and caching are task 020; this migration still
uses the provisional Core Graphics painter and makes no performance improvement claim.
