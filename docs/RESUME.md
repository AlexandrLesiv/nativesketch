# Resume Native Sketch architecture work

Handoff written 16 October 2026 (Europe/Kyiv). Read this first in a new chat.
The owner stopped implementation to conserve account usage and requested a draft PR.
This document records a stopping point, not acceptance of unfinished work.

## Subsequent owner decision — read before resuming

On 2026-10-16 the owner requested updating the plan to replace the custom binary command
protocol with typed FFI and generated declarations, while preserving shared Rust behavior.
Follow [the new 022 migration](architecture/022-typed-bindings-migration.md), slices 022A–F,
and [agent workflow](quality/agent-workflow.md). This supersedes packet-layout/adoption
instructions below and in older 022 designs; their behavioral/provider guarantees remain.
This follow-up changes documentation/instructions only, not runtime code or prior results.
Do not continue expanding v2 packet consumers. First inventory current consumers, prepare
the concrete typed contract and obtain the specified owner review. Existing working paths
remain until parity and per-host retirement gates pass. Remaining implementation, native
verification and copy-policy gaps below are still unresolved, subject to the new sequence.

The owner also requested android_ui and ios_ui implementation roles, now defined in
.codex/agents/. Task 009 uses ios_ui, 010 uses android_ui, and 022 includes both.
ui_ux_designer coordinates platform-specific briefs/Figma variants and running-app reviews;
new pages/tools still require explicit owner approval before implementation. Shared Apple
files have one lead-assigned writer; Linux/web and TV/Watch work remain explicitly assigned.
No concurrency increase or new platform acceptance is implied by these instruction changes.

## Repository and authority

- Primary checkout: `/Users/oleksandr/projects/native-sketch`.
- Branch: `codex/architecture-quality-plan`; implementation snapshot: `198996f`.
  Later commits may be this handoff only; inspect Git before changing anything.
- Draft PR: [#4](https://github.com/AlexandrLesiv/nativesketch/pull/4), targeting `main`.
  All integrated implementation was pushed; the checkout was clean at the stop.
- Required ancestry: `codex/github-pages@712059d5`, NOT `codex/renderer-results`.
  The main-branch diff also contains the unmerged prototype/Pages ancestry. Do not
  recreate the branch or import the excluded renderer-results work to fill gaps.
- No merge, release or owner task acceptance occurred. Keep PR #4 a draft until its
  outstanding verification is resolved. Owner alone approves merging into main.
- All implementation/review agents stopped. New chats should create fresh scoped agents;
  previous conversation agent names are historical, not a durable execution service.

## What the owner wants preserved

Skia is selected, including full CanvasKit on web. Do not reopen Vello comparison or
introduce a minimal-feature Skia build. Keep native controls/layout and shared Rust tool
IDs, availability, commands, state, persistence and document semantics. The intended path
is system input/native control → shared typed command → action/state → rendering. Future
agents use the same command boundary; do not implement agent transport now.

Every handwritten source/test/script/style file must be at most 200 physical lines.
Use composition and explicit ownership/state machines, not arbitrary numbered file splits.
Use pinned native Node TypeScript `.ts`, Oxc/Vite and relevant language gates; do not add
new shell, PowerShell or Python orchestration. Retire old wrappers only after host parity.
Keep existing color controls; new features or new controls require explicit owner approval.

Read current AGENTS.md, then docs/architecture.md, docs/quality.md and
[the refactoring plan](refactoring-plan.md) as needed. Lead + rust_engine own contracts;
designer owns native interaction review; fresh reviewer assesses substantive changes.
At most three subagents/two implementation writers, one feature in flight. Concurrent
writers need isolated worktrees and explicit ownership. No recursive delegation.

## Permissions and desktop use

Background builds, tests and project-scoped development setup were authorized. The owner
then prohibited Finder/app-control actions that cause permission prompts while absent.
They have returned, but that is not a new desktop reservation. **Ask before reserving the
desktop for native UI tests or benchmarks; do not reuse the previous 10-hour reservation.**
Do not launch Finder, change Accessibility/security settings, grant trust, sign in accounts
or alter global tools to bypass a block. Continue background work when a native check blocks.
No native automation or GPU measurements should run concurrently on their host.

The earlier disposable Android device approval was for `NativeSketchDesktop`; do not
identify it solely by `emulator-5554`, which another session previously used. Reconfirm
device identity and current foreground permission before any eventual device run.
Physical pen-to-screen latency remains pending a suitable camera. Software timings are
not a substitute. The Cintiq Pro 22/Pro Pen 3 was available during earlier work; recheck it.

## Implemented versus unfinished

| Area | Implemented / verified scope | Remaining |
| --- | --- | --- |
| Rust architecture, tasks 018/021 | Decomposed editor, commands, transactions, persistence/recovery, diagnostics, portable application/transport contracts | Per-host adoption and recovery/provider acceptance; do not infer every platform is migrated |
| macOS, tasks 019/020 | Native AppKit controls/shared commands; production Skia Ganesh/Metal; background checks and earlier scoped native/designer evidence | Final native regressions/designer inspection and valid release presentation measurements after timestamp/retry repairs |
| Web | Shared portable Rust/Wasm owner, full CanvasKit, bounded paths/caches, context recovery and shared f64 viewport preparation | Dedicated engine Worker/performance work, Safari/device/accessibility acceptance and remaining host matrix |
| Android | Decomposed native Views host; shared top-left scale; Rust JNI and pure Kotlin transport/JVM tests | Final checkpoint qualification/review, copy-file policy, Activity/provider adoption, legacy C++ removal, Skia, native SAF/input/focus/recreation/ChromeOS acceptance |
| Linux | Decomposed GTK host; portable owner/provider integration, chooser policy, 17 adapter/provider tests | Native Linux UI/packaging and Skia adoption; Cairo remains interim |
| Windows | Decomposed C#/WinUI host, cross-compilation/format evidence | Portable facade/Skia integration and actual Windows runtime, input, packaging and process ownership checks |
| Apple mobile/viewers | Decomposed UIKit/vision/TV/Watch prototypes, existing source policy | Portable transport/file/source contracts and adoption, appropriate Skia/viewer adapters, actual platform tests |
| Quality, task 023 | Size/dependency/lint gates, test registry, background safety lanes, evidence preservation | Fuzz failure, wider sanitizer/accessibility/endurance/performance and host-specific gaps |
| Workbench, task 025 | Local GUI, docs/tasks/dependencies/features/test evidence, background actions, Wasm preview, shared standalone export, Zed project tasks | Actual Zed interaction and speech checks; no general custom Zed panel API claimed; no new CI rollout |
| Final acceptance, task 024 | Available evidence packet and draft review | OPEN: full platform/native/package/upgrade/distribution acceptance and owner approval |

Task board `waiting`/`review` status is not proof implementation is absent or accepted.
Downstream local work was authorized while native prerequisites were blocked, but the
owner-acceptance dependency gates were not waived.

## Exact last checkpoint: Android JNI

Read [the JNI checkpoint](evidence/022-android-jni.md) and
[frozen transport contract](architecture/022-android-jni-contract.md).

- Rust worker `052c88dd7e1727c9b560d991ddaca118f9dd6617` → integrated `9b93588`.
  Fresh independent review cleared this scope. Four focused Rust tests rerun; exactly
  13 production JNI exports; no test helper in production. Safe bridge unchanged.
- Kotlin worker `00813d6` → integrated `c677991`. Includes Kotlin ownership/transport,
  Gradle JVM lanes, real tests, Android packaging helper and evidence runner changes.
  **Final Kotlin/reporting review was not completed.**
- `198996f` added the handoff. Immediately before it, the integrated checkout passed
  nine focused JNI/finalization tests; log: `artifacts/evidence/jni-pr-checkpoint-tests.log`.
- Actual development JVM runs passed 12 production cases + one separate pending-exception
  case; two pure holder cases and Android quality passed. These preceded final reporter
  edits. They are NOT exact-final-commit qualification.
- Final combined integration, final JVM rerun and dual-ABI APK build/inspection were NOT run.
  Android Activity still uses legacy JNI/C++ and Android Canvas. Do not delete them yet.

The reviewer had not verified closure of these reporter findings at the stop:
publication ordering, cancellation after child exit, failed XML retention, reused output
directories, exact compiled test/signature/export validation, source/artifact stability,
and packaging identity-read failures bypassing failure reporting.
The final worker says these are repaired: shared single-record `persistJson`, non-certifying
metadata first and execution last, explicit failed qualification. **Verify the final diff.**
The common helper `scripts/evidence/finalize.ts` changed; test existing callers too.
Do not equate metadata `checksPassed` with normalized execution certification.

## Resume sequence

1. Confirm branch/clean status and attach existing PR #4 to the new chat. Read AGENTS and
   this handoff. Preserve existing worktrees and all raw failures; do not restart from main.
2. Assign a fresh reviewer to final Kotlin/JNI/reporting changes, especially the findings
   above. Reproduce focused tests and actual JVM execution at a frozen clean source.
3. Run the full background integration gate. Build both Android JNI ABIs and inspect APK
   ELF machine, exact defined exports and embedded library hashes. Preserve retained legacy
   libraries deliberately; this is additive packaging, not Activity migration.
4. Add feature mappings for final runtime-discovered JNI IDs; normalize actual source,
   JVM/library/classes and host platform. macOS JVM evidence is not Android E2E.
5. Implement the shared copy-document prerequisite described below, with Rust ownership
   and fresh review; only then migrate Android controls/provider/session retention to it.
6. Continue Android Skia/native acceptance and Apple/Windows/Linux contracts/adoption in
   scoped checkpoints. Ask for desktop reservation when concrete native checks are ready.
7. Refresh task 024 evidence, designer/fresh reviewer assessments and package acceptance.
   Keep unsupported devices/physical latency explicit. Update PR #4; never merge yourself.

## Next shared policy design: not implemented

Read [copy policy](architecture/022-copy-document-policy.md),
[Android adoption](architecture/022-android-portable-adoption.md) and
[Apple adoption](architecture/022-apple-portable-adoption.md).
`AlwaysChoose` changes chooser routing but still retains bindings; that does not preserve
Android's completed-import/save lifetime. Do not silently retain URI access indefinitely.

The Rust specialist proposed immutable application-level
`FilePolicy::BoundDocument { save_destination: SaveDestinationPolicy } | EditableCopy`.
Store it with EditorSession and preserve across New/Open. Old constructors stay bound.
New opcode 14 has 56-byte body: existing capabilities/tool/brush followed by policy
0=bound/reuse, 1=bound/always-choose, 2=editable-copy and zero reserved u32. Old opcodes 1/8
remain unchanged; opcode 8 must still reject tag2. Add purpose tag5 `SaveCopyConfirmed`,
without changing existing record sizes or tags1–4. Freeze final layout before implementation.

- Private core gate: Idle → Choosing(exact EffectToken) → Issued(exact SaveToken) → Idle
  only on validated owner Apply. Coordinator completion/worker exit/timeout does not clear it.
  Enforce below routing, including direct core admission, alternate purposes and a second
  coordinator. Reject bound save purposes in copy mode and copy purpose in bound mode.
- Validate and prepare bounded committed snapshot/queue/output reservations before consuming
  the chooser or cancelling ink. Rejection preserves the exact continuation, draft/predictions
  and caller resource ownership. Checked fallible allocations do not promise recovery from
  arbitrary process OOM/allocator abort. Matching chooser cancellation alone clears Choosing.
- Replace public by-value `acknowledge_save` with
  `acknowledge_save(&mut Option<SaveCompletion>) -> Result<Acknowledgement, StorageError>`.
  Err preserves proof/gate/state; Ok consumes once without subsequent fallible allocation.
  No consuming compatibility wrapper. Update routing dispatch/navigation, core storage tests
  and service worker tests; preserve native-v1 ABI and bound acknowledgement behavior.
- Exact proof validation must happen before take/state mutation. Rejected copy Save payload
  in `apply_ready` must retain Ready/resource; generic stale-candidate disposal is unsuitable.
  New/Replace/install cannot bypass nonidle gates. Detach cannot discard an obligation.
- Written acknowledges issued revision only; older revision stays dirty and cannot authorize
  guarded replacement. Guaranteed-no-write failures preserve saved knowledge. Uncertain and
  cancelled-written results conservatively invalidate clean knowledge, never navigation.
  This deliberately may dirty an unrelated prior copy; no cross-session/external-write tracking.
- Import is clean/editable/unbound. Save never installs destination/bound_resource. Release
  after actual termination and accepted Apply/retirement, exactly once. Track newly acquired
  versus pre-existing/shared URI grants; never revoke a pre-existing grant indiscriminately.

Required adversarial tests include every gate phase through Ready-before-Apply, wrong/stale/
replayed proof, failure before chooser consumption, older save, cancellation/detach, alternate
write routes, exact resource release, old desktop regressions and real C/Wasm new-opcode cases.
UIKit additionally needs capture-before-picker staged export. TV/Watch need validated
candidate → atomic cache → generation-checked publication and portable source comparison;
copy-file mode alone does not solve these. Watch remains phone-preferred with eligible
same-identity network fallback; TV/Watch remain read-only.

## Toolchain and safe background commands

Use project Node **24.22.0**, not the host's different default. Existing cache locations:

```sh
export NS_REPO=/Users/oleksandr/projects/native-sketch
cd "$NS_REPO"
export PATH="$NS_REPO/.build-cache/node/node-v24.22.0-darwin-arm64/bin:$PATH"
export DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer
export JAVA_HOME='/Applications/Android Studio.app/Contents/jbr/Contents/Home'
export GRADLE="$NS_REPO/.build-cache/toolchains/gradle-9.3.1/bin/gradle"
export GRADLE_USER_HOME="$NS_REPO/.build-cache/toolchains/gradle-user"
export ANDROID_NDK_HOME="$NS_REPO/.build-cache/toolchains/AndroidNDK16248370.app/Contents/NDK"
export RUSTUP_HOME="$NS_REPO/.build-cache/toolchains/rustup"
```

Check installed SDK location before setting ANDROID_HOME; existing default is
`~/Library/Android/sdk`. Android builds use scoped
`RUSTUP_HOME=$NS_REPO/.build-cache/toolchains/rustup`, with both ARM64/x86-64 targets already
installed at the stop. Rust was 1.99.0. Do not infer target absence from default user rustup.
Do not change global defaults or reinstall existing toolchains unnecessarily.

```sh
node --test scripts/android-jni/*.test.ts scripts/tests/evidence-finalization.test.ts
node scripts/check.ts
cargo build --locked -p sketch-app-jni
cargo build --locked -p sketch-app-jni --example pending_exception_probe
node scripts/test-android-jni.ts \
  --library "$NS_REPO/target/debug/libsketch_app_jni.dylib" \
  --probe-library "$NS_REPO/target/debug/examples/libpending_exception_probe.dylib" \
  --native-source "$NS_REPO"
node scripts/build-android-portable-jni.ts --native-source "$NS_REPO"
```

Verify actual output paths first, especially if CARGO_TARGET_DIR is set. Probe and production
must run in separate JVMs; never share handles between their independent registries.
The runner creates a unique evidence directory; never reuse an existing --output path.
Commands above are resume instructions, not claims they ran at final HEAD.
Background test servers need loopback access; sandbox EPERM is infrastructure failure, not
an application regression. Use authorized scoped access without opening desktop apps.

## Evidence and process traps

- Full gate uses negative fixtures that temporarily mutate checkout files. Run exact-source
  browser/JVM/evidence qualification separately, not concurrently with that gate.
- Workbench was serving http://127.0.0.1:8085/ at the stop. Check current process ownership;
  do not assume old session IDs survive a new chat. Stop its own service before actual-service
  workbench E2E; singleton lock conflicts are intentional. Never delete a live lock.
- Restart only if needed with `node scripts/workbench.ts serve --port 8085` after assets are
  built. Foreground actions are disabled. Existing standalone export:
  `artifacts/workbench/report-shared-viewport.html` (historical snapshot, not current coverage).
- Workbench coverage reads global artifacts/evidence/discovery.json and results.json, not
  every scoped packet recursively. Missing/stale rows are honest until refreshed; do not
  copy old results under a new source revision.
- Final source inventory had zero oversized files at earlier gates, not automatic proof for
  future commits. Final Kotlin tests were added to normal tooling globs.
- Native macOS release attempts 2–4 have zero presentation timestamps despite old raw
  valid=true. Retained validity audits mark them INVALID; never reuse negative latencies.
  Timestamp/retry fixes have background/static evidence; final native verification pending.
- Fuzz at f8bb00b completed 25,000 inputs per target then failed LeakSanitizer, with an empty
  runtime reproduction. Not passing fuzz acceptance. Earlier ASan25266d5 was invalid because
  flags were shadowed; corrected df2647f proved six legacy FFI tests + deliberate canary only.

## Evidence map and local worktrees

Start with [background acceptance](evidence/024-background-acceptance.md), then scoped
[Mac Skia](evidence/020-skia-macos.md), [web CanvasKit](evidence/020-web-canvaskit.md),
[viewport](evidence/020-shared-viewport.md), [adapters](evidence/022-adapters.md),
[JNI](evidence/022-android-jni.md) and [workbench](evidence/025-project-workbench.md).

Important passing points: full background df156a7; web ea0ab9e (29 direct/17 browser,
33 mapped rows); workbench9124ec5 (17 service/14 browser,21 of23 rows; Zed/speech missing);
top-left df156a7 (six Rust + actual C/Wasm,four rows). Do not call these final-head passes.
Local ignored artifacts contain raw reports/hashes; they are not uploaded by a Git push.

Reusable worktrees below were clean/frozen; inspect them before reuse and preserve branches:

| Path under /Users/oleksandr/.codex/worktrees/ | Branch / worker revision | Integration |
| --- | --- | --- |
| skia-scene/native-sketch | codex/android-jni / 052c88d | 9b93588 |
| platform-android/native-sketch | codex/android-jni-kotlin / 00813d6 | c677991 |
| platform-linux-web/native-sketch | codex/web-viewport-preparation / 3e02a70 | 6e0c4e1 |

Rust/JNI raw copies: artifacts/evidence/022-jni-rust-worker and 022-jni-kotlin-worker.
The latter preserves android-jni/2026-10-16T06-45-43.277Z (UTC directory naming), preceding
final reporter edits. Many historical worktrees also exist; do not delete or reset them as
cleanup. Use `git worktree list` and managed attachment tools when assigning new writers.

## Suggested opening prompt for the next chat

> Resume the Native Sketch architecture/refactoring work on codex/architecture-quality-plan.
> Read AGENTS.md and docs/RESUME.md first. Continue draft PR #4 from its current pushed head;
> do not restart from main or use codex/renderer-results. Start by reviewing and qualifying
> the frozen Android JNI/Kotlin checkpoint, then follow the documented copy-file policy and
> remaining platform migration sequence. Use scoped specialists and isolated writers. Ask
> before desktop/native tests or benchmarks; preserve failed evidence and explicit gaps.
> Do not merge, release or mark task024 owner-accepted.
