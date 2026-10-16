# Task 018 Rust extraction evidence

Phase A starts at `8ae87622dc26bf7f6b5cd183c20ebc6527f88b62`, frozen by its isolated
writer at `2c9e2d405db481ef473b3f6e31d6c7c110d802e9`, integrated as
`3e01d8e`. [Ownership and diagrams](../architecture/018-extraction.md) describe the
concrete modules. Phase B is separate; this record does not certify the entire task.

## Independent compatibility review

A fresh reviewer built immutable baseline and candidate archives, then compared the public
Editor API after 32 deterministic seeds × 256 operations: 8,192 observations of results/
errors, state, document, export bytes/revisions and drawing commands. The complete traces
are byte-identical: 23,874,101 bytes each, SHA-256
`e83f1c1c41c7cf8be653b33df1fe6fee8e18348fec153f19b7ee9b5790ef99a0`.

The reviewer independently passed all 35 Rust tests, rustfmt, strict Clippy, actual Wasm
20 fixed protocol pairs and native-byte conformance. All 25 linked C exports and ten
Wasm exports remain unchanged. All 32 original test leaf names survived extraction;
27 current manifest library-test references resolve to actual discovered IDs. Fixed
document/protocol fixtures and the C header remain unchanged. No actionable findings
remained. This is finite generated-sequence coverage, not exhaustive fuzzing or performance
evidence; no native UI/physical input/other-OS/sanitizer assurance is inferred.

Reproduction instructions, oracle sources, complete traces, discovery and binary/source
hashes are preserved under ignored `artifacts/verification/018-independent/` in the lead
checkout. The README records its original temporary archive paths and exact revisions.

## Integrated checks

At clean `3e01d8e`, on macOS 27.0.1 arm64 with Rust 1.99.0 and Node 24.12.0:

- `node scripts/check.mjs`: 40 Node tests, 35 Rust tests, formatting/lint/types, dependency
  and size gates, actual Swift application linking and Rust interoperability smoke passed.
- `node scripts/check.mjs --platform wasm`: native byte equivalence and all 20 literal
  request/response fixture pairs passed.
- `node scripts/check.mjs --platform web`: actual Wasm adapter, Vite production build and
  Pages-relative asset/precache checks passed.
- `node scripts/check.mjs --platform apple-viewers`: Swift host source callback, HTTP,
  cache/ordering/fallback and viewer adapter replays passed. These are not device UI tests.
- Actual executable discovery/results passed scoped COMPAT-001 through COMPAT-005 and
  INK-003 gating; 147 report rows, six passing cases, no mapping errors. Other rows retain
  gaps. Historical baseline discovery files were not rewritten to conceal module moves.

All 65 Rust source/test files in the three crates are at most 164 physical lines.
Six oversized native/web application files remain assigned to later extraction tasks.
At that checkpoint native UI execution was blocked by console ownership, as recorded in task 017.
Typed command/capability APIs, storage effects, native presenter adoption and Skia production
integration are not delivered by phase A. No main merge or release occurred.

## Phase B: typed editor session

The writer froze `b10148f6a52fcbeccab2d66cd0cb31f7576efccf`, integrated as `b4daea1`.
The additive `EditorSession` owns the legacy editor and exposes typed commands, queries,
validated settings, capability-aware availability and correlated outcomes. Session and
interaction identities reject stale work; immutable committed snapshots exclude active
ink/predictions. Cheap view-state queries do not encode or traverse the document.
Existing native, C ABI and Wasm callers retain their behavior until host migration.

A fresh reviewer independently passed all 51 Rust tests, rustfmt, strict Clippy and the
source/dependency gate. All 16 new manifest test references matched executable discovery.
An independent 30,000-command/batch probe verified that all 20,571 rejected operations
preserved the complete view state, scene and committed document. No actionable findings
remained. This is finite generated coverage, not exhaustive fuzzing. Probe source, hashes,
logs and exact revision are under ignored `artifacts/verification/018-session-independent/`.

At clean integrated `88d49a7`, `node scripts/check.mjs` passed 43 Node tests, 51 Rust tests,
formatting, type-aware lint, type checking, dependency/size gates and the Swift application
link/interoperability smoke. A prior integration check identified three unawaited Node
test registrations; `88d49a7` repairs those and the complete check passes. The writer also
passed actual Wasm/native conformance, 20 fixed protocol pairs and the web build at its
frozen revision. These checks do not certify native UI interaction or other operating systems.

All 85 Rust source/test files are at most 185 physical lines. Six legacy host files remain
oversized. Storage effects, diagnostics, host adoption and production Skia integration
remain subsequent tasks. Console ownership is now resolved; native E2E development and
its retained failed attempts are tracked separately in task 017. No main merge or release.
