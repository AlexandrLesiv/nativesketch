# Quality and evidence

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
emulator. `scripts/run-ios <UUID> --integration-test` runs the UIKit replay in that
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

