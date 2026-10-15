# Tooling and script migration

Selected tool families, not an installed/pinned toolchain. Task 016 verifies current stable
compatible versions, records exact pins and enables checks. Do not claim latest/best from
an old conversation; review upstream compatibility when installing/upgrading.

| Scope | Formatting / analysis | Testing |
| --- | --- | --- |
| Rust | rustfmt, Clippy, cargo-deny | Cargo tests/docs, nextest where useful, proptest, targeted cargo-fuzz/Miri |
| Swift | toolchain swift-format, nonoverlapping SwiftLint, concurrency diagnostics | Swift Testing for new appropriate unit tests; XCTest/XCUIAutomation for UI |
| Kotlin | ktlint, Android Lint, compatible detekt | Native unit/adapter tests; Espresso/UI Automator |
| C# | dotnet format, Roslyn/SDK analyzers, nullable checks | Native unit/interop runner; FlaUI/UIA3 proof then E2E |
| Web/Node JS/TS | Oxlint including type-aware rules, Oxfmt, explicit type checks | Vitest for JS logic; Playwright for built web app |
| C/C++ owned bridge | clang-format, clang-tidy, compiler diagnostics | JNI conformance, applicable sanitizers |
| Retained Python analysis | Ruff; useful module type checks | Existing analysis fixtures and numerical equivalence |
| Config/data | Oxfmt supported formats, schemas | Board/manifest/config validation |

Web uses Vite 8/Rolldown/Oxc, plain TypeScript/HTML, Rust/Wasm and standard CanvasKit.
No UI framework is required. Prefer Oxfmt's npm distribution for full format support.
Keep a compiler type-check initially; evaluate Oxlint integrated type checking against
Wasm declarations before removing duplication. Retain ESLint only for a demonstrated
required rule gap, not a second complete duplicate rule set. Vitest is not browser E2E.
Vite/tool installation size is not shipped app size.

## Node automation

Use `.mjs` with JSDoc/type checking, one pinned Node LTS, explicit dependencies, Oxlint/Oxfmt.
One small entry point dispatches to modules for build/check/run/E2E/result aggregation.
Examples below are TARGET commands, unavailable until task 016 implements them:

```text
node scripts/check.mjs
node scripts/build.mjs --platform macos
node scripts/test-e2e.mjs --platform windows
```

Use filesystem/path/process APIs, executable + argument arrays, explicit working directories,
timeouts, failure codes, cancellation and child cleanup. Avoid shell-string pipelines.
Handle Windows vendor `.cmd`/`.bat` launchers in one tested adapter if required; no owned
PowerShell implementation. Native Cargo/Xcode/Gradle/.NET tools remain dependencies.
Test paths with spaces/non-ASCII, failed subprocesses, missing tools, interrupted execution
and cleanup. Do not parallelize native UI/GPU jobs on the same device.

Inventory the selected `codex/github-pages` base. Do not import renderer-results tooling.
Move routine Python task-board/server/orchestration functions to Node. Historical Python
statistics/image analysis may remain only if present/needed; a rewrite must match existing
fixtures and decisions. Vendor/generated shell code is not owned scripting policy debt.
Keep current scripts callable until replacements pass parity on applicable real hosts;
then update CI/docs/agent references and remove owned shell/PowerShell entry points.

## Enforcement details

After formatting, count all physical lines including blanks/comments in handwritten source,
tests and scripts. Exclude generated/vendor files, lockfiles/data explicitly. New/extracted
files <=200; old oversized touched files cannot grow and require a linked extraction task.
Inventory debt by file/revision/count; final refactor has zero source violations. No arbitrary
numbered splits or compressed statements. Documentation prose is not application source.
Embedded executable code in maintained HTML is source, not a data exclusion.

Dependency gates enforce core independence from native UI/Skia/platform storage, and restrict
FFI access to bridges. Prefer compiler/module boundaries and structured dependency inspection
over brittle regex-only checks. Test the gate with a deliberate violating fixture.
Pin tools/SDKs/lockfiles; run read-only checks in CI. Review suppressions, dependency/license
advisories and secret findings explicitly; do not hide existing debt in blanket exclusions.

## References checked 2026-10-15

- https://vite.dev/blog/announcing-vite8.html
- https://oxc.rs/docs/guide/usage/linter/type-aware.html
- https://oxc.rs/docs/guide/usage/formatter.html
- https://nodejs.org/api/child_process.html
- https://github.com/swiftlang/swift-format
- https://detekt.dev/docs/introduction/compatibility/
- https://github.com/EmbarkStudios/cargo-deny
