# 012: Browser Rust/Wasm editor production port

Status: waiting. Roles: ui_ux_designer, rust_engine, build_reliability, reviewer.
Dependencies: 005, 022.

Continue semantic HTML/TypeScript and shared Rust/Wasm with standard CanvasKit (Skia).
Use Oxc/Vite and Playwright. Browser controls are semantic HTML, not AppKit/WinUI widgets.
Verify the chosen CanvasKit graphics capabilities on each browser; no WebGPU or
unimplemented fallback is assumed.

Acceptance:
- Produce a browser/version/capability matrix for the pinned CanvasKit build; measure compressed payload, initialization and first usable canvas.
- Validate offline cold start, cache update/rollback, document persistence and recovery under quota/private-mode failures.
- Verify file picker permissions/cancellation, honest download-copy dirty semantics, corrupt imports and byte interoperability.
- Test pointer capture/loss, real pen/touch, keyboard editing and text-field isolation; inspect narrow/large-text layout and screen-reader behavior.
- Measure rendering/context loss and large-document limits on representative hardware; preserve fallback only if actually implemented/tested.
- Deliver reproducible browser tests, designer acceptance and fresh review. Hosting/publishing remains owner-approved.

Existing evidence: docs/evidence/005-native-platforms.md. The current browser prototype is not proof of an offline production release.

## Architecture refactoring acceptance

Follow [the selected architecture](../docs/architecture.md) and task 022 contracts.
Skia is selected; task 002 historical results are not a new implementation prerequisite.
Use native presenters/input adapters and <=200 physical lines per handwritten source/test.
Map acceptance/invariant IDs to executable tests and actual revision/platform results.
Run the applicable [native E2E and quality lanes](../docs/quality/platforms.md); injected
replay, screenshots and cross-host compilation do not replace real UI acceptance.
Preserve viewer versus editor capabilities, structured diagnostics and failure recovery.
Native UI/GPU runs are exclusive; missing hardware or unsupported automation remains a gap.
