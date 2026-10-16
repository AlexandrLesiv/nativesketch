# Task 023: background tooling probes

These are executed, project-local feasibility probes, not installed repository quality
lanes or all-platform acceptance. No global toolchain/default or security setting changed.
Host: macOS 27.0.1, ARM64. Exact logs and identity are retained in
[the probe artifact directory](../../artifacts/evidence/023-tooling-probe/identity.json).

| Probe | Source | Result and scope |
| --- | --- | --- |
| cargo-deny 0.20.2 advisories | `30bb46f` | Passed, zero errors/warnings, all workspace features, locked dependencies, no advisory ignores |
| Miri | `30bb46f` | All six existing `sketch-ffi` library tests passed with default isolation |
| AddressSanitizer | `0a43371` | Same six tests passed; Rust bridge/core/std instrumented, leak detection enabled |
| License inventory | `0a43371` | Collected package metadata; this is not an approved license allowlist or license compliance gate |

cargo-deny's crate checksum is
`e528dfcbe739af7ce37a77d3d6df1b29dd6887b1c701d888820c0f16b864f737`.
The RustSec database revision was `edb58f5974b83eb8302332821cb6d389777aa351`.
Miri and sanitizer use project-local `nightly-2026-10-15`, rustc
`1.101.0-nightly (7df81381d 2026-10-14)`; the normal compiler pin remains unchanged.

Reproduction settings: project `RUSTUP_HOME=.build-cache/toolchains/rustup`; separate
`target/miri` and `target/asan` caches. Miri executes
`cargo +nightly-2026-10-15 miri test --locked -p sketch-ffi --lib`.
The sanitizer executes `cargo +nightly-2026-10-15 test --locked -Zbuild-std
--target aarch64-apple-darwin -p sketch-ffi --lib` with
`RUSTFLAGS=-Zsanitizer=address` and `ASAN_OPTIONS=detect_leaks=1`.
The advisory probe config and JSON output are retained alongside the logs.

Neither Miri nor this sanitizer run covers Skia, JNI, Wasm, native UI, other operating
systems or the new portable facade. Remaining work includes repeatable Node runners,
bounded fuzz/property lanes, applicable native profiling/accessibility and host evidence.
No unsupported sanitizer result, universal performance budget or physical latency claim
is inferred from these probes.
