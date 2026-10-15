# Native TypeScript automation migration

Owned automation is now `.ts` ES modules, executed directly by pinned **Node 24.22.0 LTS**.
Stable **TypeScript 7.0.2** remains the separate strict no-emit compiler gate. Node strips
syntax; it does not check types. No ts-node, tsx, bundling or transpilation step is involved.
Use named types/interfaces, explicit `.ts` imports and `import type` for type-only bindings.
Compiler flags prohibit nonerasable constructs, and Oxlint rejects explicit `any` and
suppression comments. Browser output/service-worker JavaScript remains JavaScript.

Scope includes scripts, Vite configuration, Node Wasm protocol/conformance probes and the
built-browser adapter probe. Native build tools and retained legacy shell/PowerShell
entry points remain; their Node probe paths were updated, not their host acceptance status.
The browser Engine's constructor parameter property became an equivalent explicit field
assignment so its real types can be shared by strict erasable automation contracts.

The native Node execution test verifies an interface/type-only import executes, a mismatched
assignment fails the compiler, and a runtime enum fails both native Node and the compiler.
Subprocess/recovery/identity tests remain real executable cases. Background checks build and
link the native app and run the headless Swift/Rust smoke without opening a window.
Native UI E2E still requires a reserved foreground session and explicit `--interactive`.

## Evidence identity and historical records

Earlier `.mjs` evidence and baseline IDs intentionally remain unchanged under `docs/evidence/`
and `quality/baseline/`: they identify what ran at those revisions. Current instructions use
`.ts`. Node executable test IDs include the new `.ts` path and are different IDs; discovery
must execute the new file. This migration does not rewrite old results into TypeScript runs.
Generated current reports identify the source commit/diff and current runner, not history.

## Runtime verification

The local official `node-v24.22.0-darwin-arm64.tar.gz` download was checked against the matching
entry in [Node's official checksums](https://nodejs.org/dist/v24.22.0/SHASUMS256.txt):
`6a9724d9b50a5f22b4816cded441b5a8d9ddae546191a5e5b7efa5a5e2ee26d5`.
It is installed only in ignored `.build-cache/node/`; no global Node or user setting changed.
The configured Pages workflow uses the same Node pin and `.ts` commands; it has not been
executed remotely as evidence of this change. Other OS native linking/UI checks remain
host-specific gaps. Local checks cover TypeScript/Oxc, source/dependency gates, core tests,
macOS headless linking/smoke, Wasm protocol/native-byte parity and the built web adapter.

## Verified revisions

The writer froze `24ba010`; `082bd14` restores a historical documentation version without
changing runtime code. A fresh reviewer independently passed 44 Node tests, 51 Rust tests,
strict TypeScript/Oxc, size/dependency checks, Wasm/native-byte equivalence, all 20 frozen
protocol pairs and the built web/precache checks. At clean `082bd14`, executable evidence
discovery produced 152 acceptance/platform rows, 11 passing, no mapping issues and overall
incomplete coverage. The reviewer found one stale native-test README command, assigned to
the native-test integration. No other actionable migration findings remained.

At clean integrated `f64a288`, the lead repeated the complete background check (44 Node,
51 Rust, all tooling gates and native Swift linking/headless smoke) and executable evidence
generation with the new TypeScript runtime and E2E-option tests. All scoped compatibility
requirements passed; the report retained 152 rows, 11 passing and the remaining gaps.
Raw logs, reviewer evidence and the official checksum list are preserved in ignored
`artifacts/verification/typescript-automation/`. Native interaction, other-host acceptance
and remote CI are separate; no main merge or deployment was performed.
