# 016: Node tooling implementation and parity

Integrated tested revision: `46ff05a` on `codex/architecture-quality-plan`, descended from
the selected github-pages baseline. Local downstream preparation is authorized; task 015
is reviewed, not owner-accepted. Task 016 remains waiting for acceptance and the gaps below.
No legacy entry point was deleted and no product editor/renderer runtime was changed.

## Implemented

Node build/check/run/board/worktree modules replace shell-string orchestration with argument
arrays and explicit cwd/environment. Pinned Node 24.12.0, Rust 1.99.0 and .NET 10.0.107;
exact npm pins/lockfile for Oxfmt, type-aware Oxlint, TypeScript and Vite 8.3.4.
See [commands and enabled/pending tools](../quality/commands.md).

Source-size migration checks include tracked/untracked handwritten sources, physical lines
and explicit legacy debt. New files above 200 fail. Baseline-descendant historical minimum
counts form a ceiling for each oversized file: regrowth fails, a repair can pass without
rewriting history, and unrelated GitHub merge parents do not corrupt the baseline. Full
Git history is required. Final mode rejects all 12 remaining legacy violations.
Cargo metadata gates direct production/build dependencies with explicit reviewed allowlists;
they do not pretend to inspect standard-library calls, all transitive behavior or licenses.

Vite preserves separate main/engine ESM output, relative Pages URLs and the current service
worker asset/version contract. The existing actual-Wasm adapter test remains unchanged.
The configured Pages workflow now installs root tools and runs Node checks with full history.
No remote workflow run or Pages publication occurred in this milestone.

## Verification

Host: arm64 macOS 27.0.1 (26A434), Swift 6.4, Rust/Cargo 1.99.0, Node 24.12.0.

| Check | Result |
| --- | --- |
| Clean `npm ci --ignore-scripts` | Pass; 27 installed packages, audit reported zero vulnerabilities at this run |
| Node typecheck, type-aware lint and formatting | Pass on new orchestration/quality modules |
| `node scripts/check.mjs` | Pass: 26 tooling tests, source/dependency gates, 32 Rust tests, actual macOS link and Swift/Rust smoke |
| Retained `./scripts/check` | Pass in worker parity run; same 32 Rust tests and native smoke |
| `node scripts/check.mjs --platform wasm` | Pass: native/Wasm conformance plus 20 frozen request/response pairs |
| `node scripts/check.mjs --platform web` | Pass: production Vite build, real Wasm adapter and asset/precache/subpath test |
| Retained `./scripts/check-web` | Pass in worker before Vite build comparison |
| `node scripts/quality/check.mjs --final` | Expected failure: all 12 known oversized files remain; final refactor is not complete |
| Strict Swift formatting lane | Fails existing style debt; explicit failing lane, not part of a passing integration claim |

Negative tests exercise real subprocess failures, missing tools, shell punctuation/Unicode
arguments, timeouts, abort, surviving descendants, output flood, Cargo environment composition,
Windows path containment, actual Cargo alias metadata, and real Git shrink/regrow/repair/merge
history. Test fixtures are disposable and cleanup is verified by the test paths.

SHA-256 of primary-checkout artifacts:

```text
NativeSketch.app/Contents/MacOS/NativeSketch
06b589273b895fcaf9ba610bf771dd6912ce12510beff41fa75338d19cb26577
sketch_wasm.wasm
119d7a2e95a2036020a775610e7f783823573ccc0569dea7f817fcbae908d2b6
main.js
1272b74999281c9ae41e5b69a44a747cf079dfbb360299b15972b7648bc8747f
engine.js
d349ff6472183bd1afc474f8ea01fe90f1b7b9623bcbe993a66da8cae2077e28
```

## Independent review and limitations

A fresh reviewer reproduced source-growth, descendant-cleanup, output-flood, Cargo-environment
and Windows containment issues, then verified their repairs. It independently ran the Node
core and web checks at the frozen worker revision `8d2df324f42e016933554297de9d0e09bca66a04`;
integration cherry-pick `46ff05a` has the same source. No outstanding findings in reviewed scope.
Later command documentation changes and the requested `codex/` worktree prefix passed the
narrow formatting/type/lint/board checks; they do not change drawing behavior.

Pending: Windows/Linux/Android/Apple simulator/device orchestration parity, Windows process
tree cleanup on Windows, native/browser E2E and remote CI. SwiftLint, ktlint/detekt, owned
C++ formatting/static analysis, cargo-deny and expanded C# analyzer adoption remain pending.
Strict Swift formatting debt needs scoped extraction/adoption. No hardware performance,
physical pen input or pen-to-screen latency was measured. Legacy scripts stay until the
relevant host passes parity; the full cross-platform refactoring plan remains in progress.
