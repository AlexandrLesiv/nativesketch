# 009: Apple editor ports: iPhone, iPad and Vision Pro

Status: waiting. Roles: ui_ux_designer, rust_engine, ios_ui, build_reliability, reviewer.
Dependencies: 006, 022.

Extend the existing UIKit prototypes into verified native editor ports, in order iPad/iPhone, then Vision Pro. Reuse shared Rust semantics and the accepted renderer; preserve native UIKit controls and system appearance, using SwiftUI only where control behavior remains correct. Vision Pro remains a 2D window unless separately authorized.

Acceptance:
- Integrate the accepted GPU backend without introducing platform-specific document/history logic.
- Inspect phone compact/large text, iPad split/resizable windows and Vision window resize; preserve committed work and cancel temporary gestures across lifecycle changes.
- Validate real Pencil/finger/palm cancellation on available hardware, keyboard focus and VoiceOver. Record unavailable hardware honestly.
- Implement/test reliable document lifecycle and a separately specified recovery policy; do not equate retained configuration state with crash recovery.
- Run same-file interoperability and corrupt/import/save failure retention against the Rust fixtures.
- Deliver signed development builds where provisioning is available, per-device evidence, designer review and fresh independent review. Store distribution remains owner-approved.

Existing evidence: docs/evidence/005-native-platforms.md and 006-apple-device-family.md. Simulator builds already exist; this task does not claim that those are device acceptance.

## Architecture refactoring acceptance

Follow [the selected architecture](../docs/architecture.md) and task 022 contracts.
Skia is selected; task 002 historical results are not a new implementation prerequisite.
Use native presenters/input adapters and <=200 physical lines per handwritten source/test.
Map acceptance/invariant IDs to executable tests and actual revision/platform results.
Run the applicable [native E2E and quality lanes](../docs/quality/platforms.md); injected
replay, screenshots and cross-host compilation do not replace real UI acceptance.
Preserve viewer versus editor capabilities, structured diagnostics and failure recovery.
Native UI/GPU runs are exclusive; missing hardware or unsupported automation remains a gap.
