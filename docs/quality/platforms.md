# Platform quality and E2E matrix

These are required target lanes, not a claim of execution. Install runners in task 017;
prove each driver with the editor launch/draw/save/reopen journey or applicable viewer
open/cache/navigate journey before expanding it.
Record source/binary identity, runner/SDK, OS, simulator/device, input route and artifacts.

| Platform | E2E choice | Additional checks |
| --- | --- | --- |
| macOS | XCTest + XCUIAutomation | Main/Thread Performance Checker, applicable ASan/TSan, Instruments CPU/allocations/Metal |
| iOS/iPadOS | XCTest + XCUIAutomation | Device memory/thermal/lifecycle; real Pencil/finger/palm; Instruments |
| tvOS | XCTest + XCUIAutomation remote actions | Focus navigation, viewer permissions, suspend/resume, idle energy |
| watchOS | XCTest UI target | Independent cached launch, phone/server failures, transfer interruption, energy/memory |
| visionOS | Supported integration tests plus simulator/device inspection | Native visionOS SDK XCUIAutomation unavailable per checked Apple docs; preserve explicit gap |
| Android | UI Automator for system workflows; Espresso for in-app Views | StrictMode, Macrobenchmark/Perfetto, supported HWASan configurations |
| Windows | Validate FlaUI/UIA3 against WinUI then adopt | WPR/WPA/ETW, managed allocation tools, targeted Application Verifier |
| Linux | Validate dogtail/AT-SPI on controlled GNOME session | Sysprof, GTK diagnostics, sanitizers, Wayland/X11/portal matrix |
| Web | Playwright against production build, Chromium/Firefox/WebKit | Browser CPU/memory tools, axe-core, Wasm payload/startup/context loss |
| ChromeOS | Android/browser suites plus Chromebook hardware | Stylus, resize, keyboard/touch, storage integration |

Swift Testing is appropriate for new unit tests, not a replacement for XCTest UI testing.
Maestro may simplify a later mobile suite, but is not one universal native desktop driver.
Native visionOS UI automation limitations must be rechecked against the pinned SDK; do not
call injected replay E2E. Watch/TV viewers have different acceptance from editor hosts.

## Platform scenarios

- macOS: display scale/refresh changes, Cintiq disconnect, sleep/wake, repeated open/close,
  input during save, focus/capture loss. Preserve field undo and keyboard selection/scale.
- iPhone/iPad: compact layout, large text, split/resizable windows, rotation, second-touch
  arbitration, coalesced/corrected Pencil input, background/foreground and document pickers.
- visionOS: 2D window resize/lifecycle, native default Select/Pan behavior and explicit ink
  activation; indirect pinch is not measured pressure. No immersive scope expansion.
- Watch/TV: view-only enforcement, offline cache, source identity/version, staged update
  while navigating, remote/crown navigation, unavailable network/phone and low-memory use.
- Android: process death, activity/surface recreation, permission revocation, multi-window,
  stylus while finger active, keyboard/mouse provenance, actual file-provider behavior.
- Windows: per-monitor DPI, display detach, sleep/resume, pointer capture/cancel, handle
  cleanup after many windows, clipboard/file-dialog and text-focus routing.
- Linux: named compositor/protocol/GPU/driver; GNOME Wayland first, X11 only if supported;
  fractional scale and portals when packaged. A macOS GTK build is not Linux acceptance.
- Web: first load/first stroke/after-idle, offline cache, quota/private-mode failure,
  background tab, context loss, compatible JS/Wasm updates and interrupted downloads.

## Accessibility and appearance

Automate semantic/control checks where supported; separately inspect VoiceOver/TalkBack/
Narrator/Orca speech, keyboard-only navigation, visible focus, large text, contrast, reduced
motion, light/dark, long localized strings/RTL and locale-specific numeric entry/filenames.
Canvas requires semantic actions and descriptions; do not announce every sample. Preserve
native touch hit areas and usable overflow in small/short windows. System Apple appearance
and Material controls must not sacrifice readability or focus. axe-core is not a complete
accessibility certification.

## Execution rules

Native automation owns its display/device session exclusively. Identify disposable devices;
worktrees do not isolate ports, simulators or GPU resources. Use readiness signals, stable
accessibility IDs and isolated data, not arbitrary sleeps or hidden direct-command shortcuts.
Collect logs/screenshot/trace on failure; preserve retries as flaky evidence. A required
unavailable host remains a gap and cannot silently pass CI.
Sanitizers have target limits and linked-code instrumentation requirements; Apple TSan is
not supported for physical iOS-family devices. Do not compare sanitized timing with release.
Android Baseline Profiles can help managed paths but do not optimize Rust/Skia native code.
Physical pen latency, pressure/tilt and palm behavior require real-device evidence.

## Sources checked 2026-10-15

- https://developer.apple.com/documentation/xcuiautomation
- https://developer.apple.com/documentation/watchos-apps/setting-up-tests-for-your-watchos-app
- https://developer.apple.com/documentation/xcode/diagnosing-memory-thread-and-crash-issues-early
- https://developer.android.com/training/testing/other-components/ui-automator
- https://developer.android.com/topic/performance/benchmarking/macrobenchmark-overview
- https://github.com/FlaUI/FlaUI
- https://learn.microsoft.com/en-us/windows/apps/performance/
- https://gitlab.com/dogtail/dogtail
- https://github.com/GNOME/sysprof
- https://playwright.dev/docs/accessibility-testing
