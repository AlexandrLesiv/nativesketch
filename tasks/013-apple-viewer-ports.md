# 013: Apple TV and independent Watch viewer ports

Status: waiting. Roles: ui_ux_designer, macos_ui, rust_engine, build_reliability, reviewer.
Dependencies: 002, 006, 007.

Complete view-only Apple TV and independent Apple Watch ports. iPhone is the preferred Watch source, a configured server is fallback for the same published identity, and validated local copies work offline. Keep editing APIs out of viewer controls.

Acceptance:
- Validate physical WatchConnectivity delivery, independent launch without reachable iPhone, cache limits, staged updates, stale/conflicting replies and recovery.
- Test tvOS focus/remote navigation, Watch Digital Crown/gestures and VoiceOver on hardware where available.
- Preserve shared Rust validation/source policy; any viewer-specific rendering choice must document hardware limits and cannot imply universal task002 performance.
- Inspect normal/large text, content margins and navigation controls without accidental document edits.
- Verify signing/provisioning/embedded companion and standalone installation where credentials permit; no publication without owner approval.
- Deliver device evidence, designer review and fresh review. Production server/authentication is task014; public fixture adapters are not authenticated cloud storage.

Existing evidence: docs/evidence/006-apple-device-family.md and 007-watch-document-sources.md.
