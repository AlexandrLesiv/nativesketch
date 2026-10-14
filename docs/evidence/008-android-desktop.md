# 008: Android desktop adaptation evidence

Date: 2026-10-14. Branch `codex/watch-sources-desktop`.
Final implementation tested: `07cd00eb8919805da0570957010be4245c77c510`.
Android specialist source revision: `def84d9f7c5fbd5fc85b9327f223f757372f398a`.
Main remains `b63a223`; nothing was merged or published.

## Implemented

One Kotlin/Material Views editor adapts at 840 dp: compact bottom controls or a
scrollable desktop inspector. The shared Rust document, geometry, history and file
format remain unchanged. Both ARM64 and x86-64 Rust/JNI libraries ship in the debug APK.
Touchscreen hardware is optional; the Activity supports resizable windows.

Keyboard commands cover undo/redo, file actions, tools, selection deletion, Fit and
zoom, with EditText isolation. Wheel pans, Ctrl-wheel zooms, Space/middle drag pans,
and secondary click opens native actions without drawing. Mouse, stylus and touch
retain distinct Rust provenance. Size changes cancel temporary gestures while retaining
committed content, history, selection and viewport center. Native content-URI opens
and single-file drops reuse bounded import and Save/Discard/Cancel guards.

Pen/Select retain native checked accessibility state with theme-based checked/outlined
appearance. System-bar icons follow the theme background. Native path rendering remains
provisional; this is not a backend commitment or a responsiveness benchmark.

## Verification

- Required `scripts/check` passed at `dbe1157`: board, Rust formatting/clippy, all
  26 workspace tests, macOS build and Swift/Rust integration. Later changes concern
  Android input, appearance and replay scripts; no Rust/Swift code changed after this run.
- Final integrated dual-ABI build passed at `07cd00e`.
- Final specialist `lintDebug` passed at `def84d9` with warnings and no errors.
- Final integrated `scripts/check-android` passed on both disposable ARM64 API 37
  emulators: NativeSketchCompact (720x1280 pixels, 420 dpi, approximately 274 dp wide)
  and NativeSketchDesktop (1280x800 pixels, 160 dpi, 1280 dp wide).
- Replay covers actual JNI edits, pressure, cancellation, corrupt import retention,
  revision save/roundtrip, Back and incoming-file Cancel callbacks, keyboard callbacks,
  EditText isolation, size-change rollback, Space-drag, Ctrl-wheel, secondary-button
  non-inking, native MIME eligibility, exported input provenance and large-Fit zoom.
- Native inspection confirmed compact labels, a manually drawn stroke, expanded
  inspector, selected-tool distinction and readable light-theme system icons.
- Fresh independent reviewer found no remaining actionable code findings in final
  Android files, inspected both runtime PASS logs and ABI pairs, and independently
  exercised launch-script success, transient recovery, hard failure and exhaustion.
  Reviewer did not independently operate the emulators.
- Designer defined interactions and reviewed source plus supplied native screenshots.
  Final desktop checked-state/contrast changes were accepted. Final compact screenshot
  recapture and short-window visual acceptance remain incomplete, as detailed below.

Host: Apple M2 Pro /16 GiB, macOS 27.0.1; Android SDK 37, NDK r30, Gradle 9.3.1.
Native device operation was exclusive to the lead. No physical device was changed.

Logs under `artifacts/source-desktop-verification/`:
`integration-android.txt`, `android-build-final.txt`, `android-compact-final.txt`,
`android-desktop-final.txt`. `android-desktop-final.png` is valid expanded-layout
visual evidence. The file named `android-compact-final.png` captured the desktop
viewer before the tab switch completed and must NOT be cited as compact evidence.
Earlier compact inspection is recorded by `android-compact-current.png` before the
final selected-tool/system-icon styling repair.

## Failures found and repaired

Independent review found Zoom out increasing tiny Fit scales, native-format MIME drops
being rejected, and mouse strokes recorded as touch. Designer review found an unscrollable
short-window inspector. All were repaired with relevant regression coverage.

Runtime replay found that the canvas could not take focus in touch mode, so Space-drag
inked. The canvas now accepts focus in touch mode and replay verifies successful focus.
Native inspection exposed clipped compact tool labels and ambiguous tool state/low-contrast
system icons; padding and native theme-state styling were repaired.

Replay originally asserted Cancel synchronously before Android delivered its queued
button callback. It now waits for callback and dialog dismissal before asserting unchanged
bytes and cleared requests. Android 37's transient package-update Activity intercepted some
launches: the script now makes bounded retries for that response, requires a clean launch
and a PASS marker, reports labelled failures, and requires an explicit test-device serial.

## Remaining limits and incomplete inspection

Android Studio closed during the final short-window inspection and shut down the test
emulators. A disposable 840x360 profile was prepared, but its launch did not yield a
completed app check; `android-short-final.txt` records an offline-device failure. Scrolling
in a short expanded window and large-font compact controls still need visual confirmation.
The final compact screenshot was miscaptured; final compact runtime replay did pass.

Physical keyboard forwarding through Android Studio was inconclusive; replay invokes the
Activity's key callbacks, not the complete OS dispatch chain. End-to-end content-provider
grants, cross-app drag/drop, configuration recreation with an outstanding grant, TalkBack,
physical stylus/trackpad behavior and actual ChromeOS/Googlebook hardware remain unverified.
x86-64 libraries were built/packaged and independently inspected, not executed on this
Apple Silicon host. No release certification or distribution is claimed.

There is no process-death recovery/autosave. Large-file import and provider writes remain
on the UI thread; provider failures can leave a partial new destination. No performance
or GPU latency claims. Task 002's permanent renderer selection remains open.

## Reproduction

Configure Android SDK/NDK/JDK/Gradle paths as described in `apps/android/README.md`:

```sh
./scripts/build-android
ANDROID_SERIAL=<verified-disposable-device> ./scripts/check-android
```

APK: `artifacts/NativeSketch-android.apk`. The test emulators are not left running after
Android Studio exited; launch a dedicated profile before replay or manual inspection.
