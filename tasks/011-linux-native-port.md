# 011: Linux native GTK4/libadwaita port acceptance

Status: waiting. Roles: ui_ux_designer, rust_engine, build_reliability, reviewer.
Dependencies: 005, 022.

Validate and complete the existing Rust GTK4/libadwaita editor on Linux itself. Preserve real GTK controls and shared Rust document semantics. GNOME integration is the initial native target; other desktops require explicit evidence.

Acceptance:
- Build and run on a named Linux host; validate Wayland first and X11 separately if supported.
- Integrate the selected renderer into the GTK surface lifecycle, resize, scaling and device-loss paths.
- Exercise actual pointer/button state, tablet pressure, keyboard focus, two-contact cancellation and accessibility with Orca where available.
- Test portal/native file dialogs, permission denial, save/open failures and portable document roundtrips.
- Validate distribution packaging in an isolated environment and document supported graphics drivers/OS baseline; publishing requires owner approval.
- Obtain native designer review and a fresh independent review.

Existing Mac GTK builds in docs/evidence/005-native-platforms.md do not certify Linux. Preserve real lost-button cancellation; do not weaken it to accommodate a synthetic input tool.

## Architecture refactoring acceptance

Follow [the selected architecture](../docs/architecture.md) and task 022 contracts.
Skia is selected; task 002 historical results are not a new implementation prerequisite.
Use native presenters/input adapters and <=200 physical lines per handwritten source/test.
Map acceptance/invariant IDs to executable tests and actual revision/platform results.
Run the applicable [native E2E and quality lanes](../docs/quality/platforms.md); injected
replay, screenshots and cross-host compilation do not replace real UI acceptance.
Preserve viewer versus editor capabilities, structured diagnostics and failure recovery.
Native UI/GPU runs are exclusive; missing hardware or unsupported automation remains a gap.
