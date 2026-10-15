# Quality and evidence

Policy updated 2026-10-15. Current scripts below remain authoritative until their Node
replacements pass parity. New quality tooling is planned, not installed by documentation.
See [tooling](quality/tooling.md), [feature coverage](quality/coverage.md),
[platform E2E](quality/platforms.md), [reliability](quality/reliability.md), and
[invariants](architecture/invariants.md). The [refactoring plan](refactoring-plan.md)
assigns implementation owners and gates.

## Required target gates and adoption

| Gate | Owner | Implementation task / scope |
| --- | --- | --- |
| Format/lint/type and <=200 physical source lines | build_reliability + implementers | 016; new/extracted files comply immediately |
| Dependency/FFI boundaries | rust_engine + build_reliability | 015/016/018; verify violating fixtures |
| Acceptance/invariant IDs and executable mappings | implementers + reviewer | 015/017; feature manifests and real runner discovery |
| Platform E2E | platform implementer | 017/019/022 and port tasks; real UI route |
| Error diagnostics and recovery | implementers + build_reliability | 021; failure injection and redaction |
| Render/cache equivalence and performance | rendering_performance | 020/023; release builds on named hardware |
| Fuzz/property/sanitizer and dependency checks | rust_engine + build_reliability | 018/023; supported configurations only |
| Accessibility and native behavior | ui_ux_designer + platform implementer | 019 and port tasks; separate automation/runtime evidence |
| Package/upgrade/symbol checks | build_reliability | 024; selected distribution, no publishing implied |

Review policy applies now. Until an automated gate exists, report its manual evidence or
gap. Do not claim automation from policy text. Existing oversized files are enumerated debt:
new/extracted files <=200; touched oversized files cannot grow and have an extraction task.
Final refactor acceptance requires zero handwritten source violations. Do not compress code,
hide executable code in data or split a coupled object into arbitrary fragments.

For each change, name acceptance cases and invariant IDs before implementation. Native
E2E, bridge/replay, core tests and physical-device checks remain distinct. Required missing,
skipped, flaky or unsupported evidence cannot be silently reported as passing. Generate a
feature/platform matrix from manifests, discovered tests and revision-specific results.
Code coverage percentages supplement this matrix; they do not prove user journeys.

Use fast deterministic checks per change; affected native E2E per integration; longer fuzz,
sustained and hardware checks in explicitly provisioned jobs or release validation. Do not
create recurring automation/runners merely because this policy describes a cadence.

Use checks proportionate to the changed behavior. Do not write tests that merely repeat
constants or implementation details. Do not rerun unrelated suites after successful checks
unless new changes or evidence justify it.

## Local baseline

`./scripts/check` validates the task board, checks Rust formatting and clippy, runs the
Rust tests, builds the macOS shell, and runs its Swift-to-Rust first-stroke integration check. Rust tests cover transactional
editing, pressure replay, persistence, and FFI failure behavior. The Swift integration
check covers geometry/style round-trips and editing through the actual C boundary.
Running-app interaction, native event replay, Windows, physical tablets, and GPU or
pen-to-screen latency are separate evidence; a green check alone does not cover them.

`./scripts/check --core` skips macOS and is suitable for a non-Mac Rust runner.
Generated artifacts are not source. Keep test fixtures synthetic and disposable.

## Portable host checks

`scripts/check-wasm` verifies the raw Wasm bridge and native byte equivalence.
`scripts/check-web` builds TypeScript and runs real-Wasm adapter/lifecycle regressions.
`scripts/check-linux` builds GTK/libadwaita, runs clippy and the native adapter test;
running this on macOS does not certify Linux display servers or portals.
`scripts/check-android` runs the JNI/input replay on the selected Android device or
emulator. Always identify the disposable test device before choosing `ANDROID_SERIAL`.
Desktop replay covers keyboard callbacks, resize cancellation, native MIME eligibility,
mouse/stylus/touch provenance and large-document Fit/zoom. Real input dispatch, small/short
window usability and file-provider permissions also require native runtime inspection.
Dual-ABI packaging alone does not certify ChromeOS/Googlebook hardware.
`scripts/run-ios <UUID> --integration-test` runs the UIKit replay in that
simulator. Coordinate device runs and native UI inspection exclusively on the host.
`scripts/check-windows.ps1` builds the x64 Rust DLL/WinUI host and runs C# conformance
on Windows. Cross-host C# compilation or interop checks do not replace Windows launch.

`scripts/build-apple-family simulator` builds all Apple hosts and embeds the Watch
companion in the iPhone app. Device variants require owner provisioning.
`scripts/check-apple-viewers` exercises actual Rust read-only imports, byte identity,
stale/corrupt/oversized transfer retention, atomic caches and HTTPS download policy.
Watch source checks additionally cover Rust version decisions, staged update ordering,
legacy migration, phone-first fallback, and actual utility-queue callback delivery to
MainActor. URLProtocol HTTP fixtures run on macOS; watchOS explicitly skips that fixture
because its runtime did not intercept the requests. Watch replay launch validates the
PASS marker as well as process launch status.
Simulator viewer replays are debug-only and exit with a result; physical WatchConnectivity
delivery and Vision input require separate device evidence.

Record OS, SDK, exact source revision, build, adapter execution, and visible interaction
as separate evidence. See `docs/evidence/005-native-platforms.md` for current coverage.

## Feature-specific merge evidence

| Change | Required evidence |
| --- | --- |
| Domain/editor behavior | Acceptance cases and relevant invariants; undo/redo where applicable |
| Input | Recorded sample replay, cancellation, missing pressure, coordinate transforms |
| Rendering | Visual fixtures and measured affected frame stages on named hardware |
| Persistence | Round-trip semantics, corrupt/interrupted writes, migration fixtures |
| Native UI | Running-app interaction checks, resizing, focus/shortcuts, accessibility |
| FFI | Ownership/lifetime/error contract and a real consumer smoke/integration test |
| Config/build/workflow | Installed-client config validation and affected build commands |

## Performance

Reference hardware and absolute budgets are OPEN. Do not invent a verified frame target.
Capture build mode, renderer revision, scene and input fixture IDs, resolution, display
refresh, machine/OS/GPU, warm-up, repetitions, and power/thermal conditions.
Report CPU and GPU timing separately, p50/p95/p99, missed deadlines, and memory.
Run performance work exclusively on its machine. Confirm regressions exceed measurement
noise. Pen-to-screen latency requires end-to-end measurement, not just GPU timing.

## Independent review and merge

Fresh reviewer examines requirement, exact diff, relevant code, and evidence. Findings
must identify a trigger, consequence, and location. Mark unverified concerns as such.
Resolve correctness/data-loss issues before presenting for merge. Minor suggestions
may be deferred explicitly. Changes to golden images, thresholds, CI, or acceptance
criteria require explicit explanation and owner review; never silently weaken checks.

Report the tested commit or working-tree diff identity. Recheck affected behavior after
integration. Human merge approval is required by project policy; remote enforcement is
not configured. Never represent an agent's agreement as proof that the implementation works.

