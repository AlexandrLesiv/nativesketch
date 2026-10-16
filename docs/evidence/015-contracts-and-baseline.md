# 015: Contracts and compatibility baseline

Implemented from `codex/github-pages@712059d5c4470c9770d08c42de8f1391884a66be`, through
planning commit `e467d402c0f9888ae11a5312a2caabcd9335480f`. No code was imported from
`codex/renderer-results`. Product runtime behavior is unchanged in this milestone.

## Delivered

- [Concrete contracts](../architecture/concrete-contracts.md): command/query/effect and
  cohesive file/type map, one mutable owner, viewer capabilities, immutable snapshots,
  errors, input batches, scene identity and save/session/request lifetimes.
- [Compatibility baseline](../architecture/compatibility.md): ABI 2, Wasm 1, document bytes
  and native interaction differences; current enforcement distinguished from target design.
- [Source inventory](../../quality/baseline/README.md): all 154 base files classified,
  67 handwritten sources, 12 oversized files linked to extraction tasks.
- `quality/features/`: 41 acceptance cases with actual test mappings or explicit gaps.
  The 26 baseline library tests and six new integration tests were discovered by Cargo.
  No native E2E coverage or passing coverage dashboard is invented.
- Fixed 216-byte document and 20 Wasm request/status/response fixtures. Six new Rust
  tests exercise prediction exclusion, transaction/history retention, atomic invalid
  batches, correction rules, ABI layouts/capacity and stale save acknowledgement.
  Existing `check-wasm` now invokes the new JavaScript replay, including in current CI.

## Validation and identity

Tested integrated code revision: `4c729f3bef9b0ecc4ec19d280c3053482fa3a4e7`.
Subsequent changes only clarify 64-bit test applicability, evidence and task status.
Host: arm64 macOS 27.0.1 (26A434), Rust/Cargo 1.99.0, Swift 6.4, Node 24.12.0.

| Check | Result |
| --- | --- |
| `./scripts/check` | Pass: board, rustfmt, strict Clippy, 32 Cargo tests, macOS build and Swift/Rust integration smoke |
| `./scripts/check-wasm` | Pass: real native/Wasm conformance and all 20 frozen pairs; independently rerun by reviewer |
| Baseline inventory regeneration | Byte-for-byte identical at 712059d5 |
| Cargo discovery / feature mappings | All 32 IDs resolve; duplicate acceptance IDs absent; missing coverage retains gap records |
| New handwritten source size | 55, 128, 83 and 50 physical lines; no oversized production file changed |
| `git diff --check` | Pass |

Swift/Rust smoke covers injected mouse/pressure geometry, cancellation, resize undo/redo,
exact style/geometry save/reopen and failed save/open retention. It does not deliver native
pointer events. Build artifact: `artifacts/NativeSketch.app`, local ad-hoc signature.
Swift links with existing nonfatal warnings about two missing CommandLineTools search paths.
The ordinary sandbox cannot run Swift's nested sandbox; the authorized host build passed.

SHA-256 of tested binaries (generated artifacts, not committed):

```text
NativeSketch.app/Contents/MacOS/NativeSketch
06b589273b895fcaf9ba610bf771dd6912ce12510beff41fa75338d19cb26577
document_compatibility-a7d233503783723a
2dcbc2cb67a871821fce70f0cd32f2b6453929a3a8270b4da5142372c13dc648
abi_compatibility-ecb4417547fa2755
4f4011fc76e659d1e9da7a2dc558388020bb7d197c462a644a43f7ef90d32a47
sketch_wasm.wasm
119d7a2e95a2036020a775610e7f783823573ccc0569dea7f817fcbae908d2b6
```

## Review and remaining work

Configured ui_ux_designer statically audited platform interactions at e467d40 before contract
freeze. Projection/radial resize, zoom limits, Escape selection behavior, focus priority,
second-touch ownership and viewer-relative zoom must remain explicit during extraction.
No visible interface change was made, so no new UI inspection is claimed.

Fresh configured reviewer independently reproduced inventory, discovery, new Rust tests,
Wasm checks and lint at the integrated revision; no blocking findings. Its nonblocking
applicability note was addressed: pointer width is now separate from platform in both
the ABI-layout mapping and discovery record.

Native E2E, unexecuted OS/C compiler layouts, production Rust viewer enforcement, asynchronous
storage ordering, renderer/cache recovery, property/fuzz testing and physical pen latency
remain tasks in the approved sequence. These are not certified by the compatibility guards.
Task 015 is ready for owner review; no main merge, publication or release occurred.
