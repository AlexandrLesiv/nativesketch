# 012: Browser Rust/Wasm editor production port

Status: waiting. Roles: ui_ux_designer, rust_engine, build_reliability, reviewer.
Dependencies: 002, 005.

Continue the semantic HTML/TypeScript host and shared Rust/Wasm engine. Browser controls remain semantic HTML; no claim of native AppKit/WinUI widgets. Renderer compatibility must be assessed separately for WebGPU and any deliberately supported fallback.

Acceptance:
- Produce an explicit browser/version/capability matrix and explain any backend divergence from task002's macOS recommendation.
- Validate offline cold start, cache update/rollback, document persistence and recovery under quota/private-mode failures.
- Verify file picker permissions/cancellation, honest download-copy dirty semantics, corrupt imports and byte interoperability.
- Test pointer capture/loss, real pen/touch, keyboard editing and text-field isolation; inspect narrow/large-text layout and screen-reader behavior.
- Measure rendering/context loss and large-document limits on representative hardware; preserve fallback only if actually implemented/tested.
- Deliver reproducible browser tests, designer acceptance and fresh review. Hosting/publishing remains owner-approved.

Existing evidence: docs/evidence/005-native-platforms.md. The current browser prototype is not proof of an offline production release.
