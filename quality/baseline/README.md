# Selected-base inventory

Source revision: `712059d5c4470c9770d08c42de8f1391884a66be` (`codex/github-pages`).
The architecture commit `e467d40` changes documentation/instructions only; source bytes
and the 26 discovered Rust library tests are unchanged from that base.

Reproduce every tracked file's category, physical line count and SHA-256:

```console
node scripts/quality/baseline-inventory.mjs 712059d5 quality/baseline/source-inventory.json
cargo test -p sketch-core --lib --locked -- --list
cargo test -p sketch-ffi --lib --locked -- --list
cargo test -p sketch-wasm --lib --locked -- --list
```

`source-inventory.json` contains all 154 tracked files: 67 handwritten source files
(10,837 physical lines), 48 documentation files, 35 configuration files, three lockfiles
and one fixture-data file. There is no tracked generated/vendor code. Source includes
HTML/CSS, Gradle Kotlin scripts, Swift package code and extensionless owned scripts.
Twelve source files exceed 200 lines; each names its planned extraction task. This is a
historical snapshot, not an allowlist for future additions or source growth.

`discovered-tests.json` records actual Cargo test discovery, not a source-text search.
It does not record execution success. The initial manifests in `../features/` map target
invariants to those test IDs and explicitly identify partial coverage. A mapping does
not imply that a test proves every clause of an invariant or a native UI journey.
Task 017 will install schema validation, fresh discovery and revision-aware result joins.

Existing Swift integration, Apple viewer/source replay, Node Wasm conformance, web
adapter replay and Windows interop programs are coarse standalone probes. They are
not listed as discoverable named unit tests or native UI E2E. Their current entrypoints:

| Probe | Existing runner | Boundary |
| --- | --- | --- |
| Swift/Rust smoke | `scripts/check` | Injected Swift C-ABI calls, no native events |
| Apple viewer/source/HTTP replay | `scripts/check-apple-viewers` | Adapter policies and local HTTP, no Watch UI |
| Raw Wasm conformance | `scripts/check-wasm` | Real Wasm messages/native byte comparison |
| Browser adapter | `scripts/check-web` | Node + compiled TS + Wasm, no browser UI |
| Windows interop | `scripts/check-windows.ps1` | C# C-ABI calls, no WinUI event delivery |

No native E2E runner was discovered at this baseline. Focus, cancellation on capture
loss, phone/network delivery, physical pen input and renderer/device loss need their
own evidence. Preserve these gaps when adding the automated coverage report.
