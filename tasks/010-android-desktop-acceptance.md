# 010: Android, ChromeOS and Googlebook port acceptance

Status: waiting. Roles: ui_ux_designer, rust_engine, build_reliability, reviewer.
Dependencies: 008, 022.

Complete native Kotlin/Material Views acceptance on Android phones/tablets and compatible ChromeOS/Googlebook systems. Continue the existing ARM64/x86-64 implementation; adapt by window/input capabilities, not branding. Verify actual runtime availability on each target before promising support.

Acceptance:
- Finish task008 short-window inspector scrolling and final compact/large-font designer checks; replace the miscaptured compact evidence.
- Verify OS-level keyboard dispatch, text-field isolation, mouse/middle/secondary buttons, wheel/trackpad, touch and physical stylus.
- Exercise real content providers and cross-app drops, temporary grants, dirty Cancel, rotation/recreation and rejected/corrupt/oversized files.
- Integrate and benchmark the accepted renderer on target hardware; execute both ARM64 and x86-64 packages where hardware is available.
- Define and test process-death recovery separately from current configuration retention.
- Inspect TalkBack, resizable/multiwindow behavior, density changes and loss/recreation of rendering surfaces.
- Deliver real-device/OS evidence, final designer assessment and fresh review. Emulator results alone do not certify ChromeOS/Googlebook.

Existing evidence: docs/evidence/008-android-desktop.md. Production network storage is separate.

## Architecture refactoring acceptance

Follow [the selected architecture](../docs/architecture.md) and task 022 contracts.
Skia is selected; task 002 historical results are not a new implementation prerequisite.
Use native presenters/input adapters and <=200 physical lines per handwritten source/test.
Map acceptance/invariant IDs to executable tests and actual revision/platform results.
Run the applicable [native E2E and quality lanes](../docs/quality/platforms.md); injected
replay, screenshots and cross-host compilation do not replace real UI acceptance.
Preserve viewer versus editor capabilities, structured diagnostics and failure recovery.
Native UI/GPU runs are exclusive; missing hardware or unsupported automation remains a gap.
