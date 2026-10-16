# Executable tooling commands

Task 016 adds a Node path alongside retained legacy entry points. Run from any directory:
entry points resolve their checkout, not the caller's current directory. Install repository
packages with `npm ci --ignore-scripts` using Node **24.22.0** (`.node-version`). Rust
**1.99.0**, rustfmt and Clippy are pinned in `rust-toolchain.toml`; .NET **10.0.107** is
pinned in `global.json`. No command changes a global default or grants folder trust.

| Command | Scope |
| --- | --- |
| `node scripts/board.ts [--check]` | Original board validation/listing, including done-dependency requirement |
| `node scripts/check.ts [--core]` | Board, JS formatting/type/lint, tooling negative tests, source/dependency gates, Rust; macOS link/smoke and GPU-free Swift scheduling policy tests unless core |
| `node scripts/check.ts --platform wasm` | Real Wasm/native document equivalence and protocol-v1 fixtures |
| `node scripts/check.ts --platform web` | Wasm, TypeScript, Vite, actual Wasm adapter and precache/subpath checks |
| `node scripts/check.ts --platform apple-viewers` | Swift/Rust host viewer/source replays |
| `node scripts/check.ts --platform linux` | GTK Cargo format/Clippy/tests/build |
| `node scripts/check.ts --platform windows` | Native Windows Rust/WinUI build and interop runner |
| `node scripts/check.ts --platform android` | Existing installed-device replay; requires verified `ANDROID_SERIAL` |
| `node scripts/test-legacy-ffi.ts [--output NEW_DIRECTORY]` | Background macOS legacy ABI 2 C compiler/link/runtime check, negative controls and exact binary discovery/execution evidence; included in the full macOS check |
| `node scripts/build.ts --platform PLATFORM` | macos, ios, tvos, watchos, visionos, apple-family, android, linux, windows, wasm, web |
| `node scripts/run.ts --platform macos` | Build and open native app |
| `node scripts/run.ts --platform web` | Build and serve production files on loopback; `PORT` defaults 8077 |
| `node scripts/run.ts --platform ios --device UUID -- --integration-test` | Install/launch exact simulator; tvos/watchos/visionos supported; Watch PASS marker required |
| `node scripts/new-worktree.ts task-name` | Clean primary checkout only, local branch/worktree from HEAD |
| `node scripts/lint.ts --platform rust` | Rust formatting and strict Clippy |
| `node scripts/lint.ts --platform web` | Node gates plus web compiler checking |
| `node scripts/lint.ts --platform swift` | Strict installed-toolchain swift-format; existing debt fails visibly |
| `node scripts/lint.ts --platform android` | Existing pinned Android Gradle Plugin Lint task |
| `node scripts/lint.ts --platform windows` | Native Windows dotnet format verification |
| `node scripts/test-evidence.ts --run` | Execute discovered Cargo tests and generate revision-bound HTML/JSON coverage; report success is not full coverage |
| `node scripts/test-evidence.ts --gate` | Require all registered acceptance rows; currently fails on documented coverage gaps |
| `node scripts/test-e2e.ts --platform macos` | Background-only app/test preparation; no executable discovery or native UI |
| `node scripts/test-e2e.ts --platform macos --interactive` | Run native XCTest journey in an explicitly reserved active unlocked desktop session |
| `node scripts/test-e2e.ts --platform macos --interactive --only xctest:NativeSketchUITests:ClassName/testMethod` | Execute one exact discovered case; repeat `--only` for a scoped batch, preserving full discovery |

See [evidence runner](evidence-runner.md) for scoped acceptance gates, Node discovery,
native imports and retained retry attempts. Native E2E defaults to background preparation;
`--prepare-only` is an explicit alias. Preparation writes blocked evidence and exits nonzero
deliberately because the native journey has not executed. `--interactive` is required for
foreground input and cannot be combined with `--prepare-only`. Normal check/build/core/bridge
commands stay in the background; the macOS smoke path exits before NSApplication starts.
Run native UI tests when verifying relevant commits or developing those tests, not on every
background check. Reserve a foreground session or dedicated host for UI tests/GPU measurements. Hidden,
minimized or off-console execution never counts as visible native acceptance.

Build flags: `--profile debug|release` for macOS; `--variant simulator|device` for Apple
family. Device builds still require owner signing/provisioning for installation. Run
commands never choose an arbitrary TV/Watch simulator. Android Gradle/JDK/SDK/NDK settings
retain existing environment overrides. Native tools must exist; missing tools are failures.
Vendor `.cmd`/`.bat` launchers are rejected; use a native executable or Node CLI entry
point. Windows orchestration uses native `cargo`/`dotnet`, not npm.cmd or PowerShell.

## Enabled versus pending

Root npm exact pins: Oxlint 1.88.0, oxlint-tsgolint 7.0.2004, Oxfmt 0.73.0, TypeScript
7.0.2, Vite 8.3.4, Node declarations 24.19.2. Lockfile integrity pins transitives. Official
npm package metadata checked 2026-10-15; their Node engine ranges include 24.22.0.
Vite 8 uses Rolldown/Oxc. The built web app preserves separate main/engine ESM files,
relative URLs and standard service-worker cache behavior. No renderer runtime changes.
The old apps/web compiler package/lockfile remain for legacy-path parity until removal.

JS formatting/type-aware lint checks cover all new orchestration and quality modules;
legacy app sources are not silently reformatted. After the reviewed Android composition, the source-size gate reports four linked
legacy debts and rejects new violations, growth after partial extraction, and all debt
in final mode. Cargo dependency checks enforce boundaries; **not** license/advisory scans.
The macOS native boundary gate rejects legacy bridge imports/linkage, manual C symbol
declarations, and legacy/test-only headers reachable from the production session module.
Its negative fixtures also exercise the actual quality CLI. This is a source regression
guard for the supported module layout, not a Swift/C parser or proof of semantic routing;
actual compiler consumers, bridge behavior tests and native journeys remain required.

Swift 6.4 toolchain `swift-format --version` reports `main` on this host. Strict lint
currently fails existing style debt (including semicolon/line-length violations), so it
is an explicit failing command, not a green integration gate. Do not autoformat oversized
source before its extraction. Selected SwiftLint rules remain uninstalled. Android now has
project-scoped ktfmt 0.64 with `kotlinFormat`, `kotlinFormatCheck`, `kotlinSourceSizeCheck`
and aggregate `androidQuality` Gradle tasks. The aggregate passed on worker `398261d`:
zero Lint errors, 15 unsuppressed warnings, JVM tests `NO-SOURCE`. The root Android lint
entry point still runs only `lintDebug`; aggregate integration remains pending.
Kotlin detekt, clang-format/tidy, cargo-deny and C# analyzer expansion remain pending
installation and applicable host verification. Windows format is unverified here.

The Pages workflow invokes Node portable/Wasm/web checks with complete Git history for
the growth gate. This is configured remote CI, not evidence of a completed remote run.
All old scripts are preserved: Mac/web parity does not certify Windows/Linux/Android or
Apple simulator/device parity. Host-specific runtime acceptance and actual UI E2E remain
separate from script, compiler, bridge and adapter tests.

## Process and test behavior

Subprocesses use executable/argument arrays, explicit directories, default ten-minute
bounds, AbortSignal and SIGINT/SIGTERM cleanup, bounded captured diagnostics and POSIX
process-group teardown. Windows tree teardown still requires native-host validation.
Fixtures live in ignored `artifacts/tests/` and cleanup executes even after failures.
Negative tests cover Unicode/spaces/shell punctuation, missing/failing tools, timeout,
cancellation, surviving descendants, output floods, environment composition and board
validation. Source/dependency fixtures intentionally fail the real gate. Web checks
validate emitted files, transitive imports, Pages-relative URLs and the cache digest;
these do not replace browser UI/offline E2E.

References: [Vite 8](https://vite.dev/blog/announcing-vite8),
[Oxlint type-aware checks](https://oxc.rs/docs/guide/usage/linter/type-aware.html),
[Oxfmt](https://oxc.rs/docs/guide/usage/formatter.html).

Automation now executes native `.ts` modules under Node24.22.0; see
[the migration contract and historical-evidence policy](typescript-automation.md).
TypeScript7.0.2 checks types separately from Node's syntax stripping.
