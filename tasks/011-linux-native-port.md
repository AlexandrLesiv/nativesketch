# 011: Linux native GTK4/libadwaita port acceptance

Status: waiting. Roles: ui_ux_designer, rust_engine, build_reliability, reviewer.
Dependencies: 002, 005.

Validate and complete the existing Rust GTK4/libadwaita editor on Linux itself. Preserve real GTK controls and shared Rust document semantics. GNOME integration is the initial native target; other desktops require explicit evidence.

Acceptance:
- Build and run on a named Linux host; validate Wayland first and X11 separately if supported.
- Integrate the selected renderer into the GTK surface lifecycle, resize, scaling and device-loss paths.
- Exercise actual pointer/button state, tablet pressure, keyboard focus, two-contact cancellation and accessibility with Orca where available.
- Test portal/native file dialogs, permission denial, save/open failures and portable document roundtrips.
- Validate distribution packaging in an isolated environment and document supported graphics drivers/OS baseline; publishing requires owner approval.
- Obtain native designer review and a fresh independent review.

Existing Mac GTK builds in docs/evidence/005-native-platforms.md do not certify Linux. Preserve real lost-button cancellation; do not weaken it to accommodate a synthetic input tool.
