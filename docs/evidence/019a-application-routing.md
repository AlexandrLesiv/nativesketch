# Task 019A: portable application routing evidence

Tested source: `a9360b3a9c36c70656b92f5807e48bd500a105ec`, isolated branch
`codex/application-command-router`, starting at `2db951c`. This is the portable semantic
router stage. It does not introduce C symbols, native runtime workers or migrated Swift callers.

Host: Apple Silicon macOS 27.0.1 (26A434), Rust 1.99.0 and Node 24.22.0. Checks ran in the
background without opening/focusing an app. Existing offline-installed Node dependencies
were retained in the isolated checkout.

## Exact-revision results

`node scripts/check.ts --core`: passed 54 Node tests and 128 Rust tests, including all 23
new routing tests. Rust format/Clippy, Oxfmt/Oxlint, strict TypeScript, source-size and dependency
checks passed. Five pre-existing host source-size debts remain; all added source/tests are
at most 171 physical lines. One inherited storage child-writer fixture is ignored in ordinary
enumeration and explicitly executed by its process-kill parents. The core runner explicitly
skips macOS build/ABI smoke; native UI and physical input are not included in these results.

`node scripts/check.ts --platform wasm`: actual Node/WebAssembly/native document-byte
conformance and all 20 frozen protocol-v1 request/response pairs passed. ABI2 and document
fixtures also passed in the Rust workspace suite. No existing protocol declaration changed.

`cargo test -p sketch-core application::routing -- --list` discovered the 23 actual executable
IDs. Raw logs/discovery are retained under ignored `artifacts/verification/019a/` in the
implementation worktree. Lead-owned acceptance mappings remain separate from this report.

| Artifact | SHA-256 |
| --- | --- |
| `core-check.log` | `64b7c0485641468fbebc7972ec219e8bb21d7815dea0473d302c4e60c887954b` |
| `wasm-check.log` | `1185e606b69ea8630651703941f28a8dc08c90c64953b66f338cea5607ff7d3d` |
| `discovered-tests.log` | `206db86cf62e971d1d32dcf01d7012fbe05bd123e68f6f73121be420b611770f` |
| `target/wasm32-unknown-unknown/release/sketch_wasm.wasm` | `326405c070f25b25d4b70f8231d74f5000da1ee972a2b7e032f4f5e20d67268f` |

## Behavior exercised

All new tests are under `application::routing::tests`. They cover portable portions of
INPUT-001, INK-001/002, UNDO-001, SAVE-001/002/003, LOAD-001, VIEW-001 and ASYNC-001; inherited
core tests retain brush/sample/transaction semantics. They are not native E2E evidence.

| Test module | Cases | Behavior |
| --- | ---: | --- |
| `effects` | 4 | Invalid/replayed effects preserve draft/pending state; wrapped direct replacement denied; capability/busy preflight; accepted file cancellation; readonly export; whole-batch rejection and one-operation undo |
| `lifecycle` | 5 | Rejected completion retains unique ownership; failed save preserves edits and reports details; source-panel late edits rejected; lost completion remains visible; cross-session effect rejection |
| `limits` | 1 | 1024 undelivered obligations cannot grow without bound or silently unblock work |
| `navigation` | 4 | Taken-but-undelivered completion cannot bypass navigation; save/close dirty rechecks; stale discard decisions; cancelled navigation does not cancel writes |
| `pending_binding` | 1 | Unbound Save cannot launch another Save As chooser or cancel draft while binding adoption is pending |
| `replacement` | 4 | Stale candidate stays owned and current state unchanged; atomic binding/history/selection reset with current settings; dirty/unbound recovery; issued preparation failure preserves content |
| `saves` | 4 | Ordinary old-binding Save during Save As; late Save blocks install until acknowledgement; checkpoint after replacement cannot clean/bind; accepted admission failure distinguished from rejection |

Draft review identified the unbound Save/pending Save As preflight gap; the guard and its
regression were included before the frozen commit. A preliminary Clippy failure in explicit
non-Drop fixture destruction was repaired without suppressing the lint. Final checks above
ran on the frozen implementation. Independent assessment is reported separately by the fresh
reviewer against this source identity.

## Ownership and remaining work

The API is documented in `docs/architecture/019a-application-routing.md`. The session owns
semantic decisions and admitted completion obligations. CompleteSave consumes trusted unique
results only after validation. Install retains rejected candidates and returns retired state.
The future native runtime must reserve retirement capacity, dispose off-owner, preserve grant
lifetimes and deliver every completion. A lost completion intentionally leaves a visible
outstanding obligation; no timeout guesses success or grants navigation permission.

Snapshot admission is still synchronous O(document), outside input callbacks. No filesystem,
thread scheduling, latency, native text-field or physical-input behavior is claimed here.
019B/C add versioned native interfaces and services; 019D/E adopt native presentation and run
actual UI journeys. No Skia/Core Graphics changes, main merge or release occurred.
