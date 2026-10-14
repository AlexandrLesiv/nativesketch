# Android native host

Kotlin Android Views and Material Components 1.13.0 controls, JNI ABI 2 transport,
and the shared Rust editor. Android Canvas paints Rust-generated paths provisionally;
this does not decide the production renderer. No network permission or runtime service.

Build with JDK 17+, Gradle 9.3.1, Android SDK 37 / build tools 36.0.0, NDK r30, and
Rust targets `aarch64-linux-android` and `x86_64-linux-android`. Set `ANDROID_HOME`, `ANDROID_NDK_HOME`, `JAVA_HOME`,
and optionally `GRADLE` to local tool locations, then run `scripts/build-android`.
The resulting ARM64 + x86-64 debug APK is `artifacts/NativeSketch-android.apk`.
`ANDROID_SERIAL=emulator-5554 scripts/check-android` installs and runs a debug-only
on-device JNI/touch replay. Coordinate this with other native UI automation.

Pen draws with finger or stylus. Pressure is present only when a stylus reports a
valid pressure axis; its device range is normalized. Select taps hit-test; dragging
the bottom-right handle uniformly scales about top-left. The Selection sheet offers
previous/next, delete, and numeric uniform scale. Two-finger pan/zoom cancels in-flight
ink/resize. Zoom sheet offers in/out/reset. Document overflow offers New/Open/Save copy.
SAF file operations suspend editing; invalid import and failed/cancelled save keep the
current document. Save acknowledges Rust revision only after the provider stream closes
successfully. New/Open and system Back unsaved prompts offer Save/Discard/Cancel. Back uses the
AndroidX activity dispatcher; cancelling preserves the editor and dirty state.

Configuration recreation retains the Rust editor, history, dirty flag, viewport center,
and pending save snapshot. Process termination does **not** recover unsaved work; save
before leaving. No autosave yet. Local Android file providers do not offer a universal
atomic replacement or durable-fsync contract, so Save a copy is the supported operation;
a provider I/O failure can leave a partial newly-created destination. The current picker
bridge performs bounded reads/writes on the UI thread with editing disabled. Large-document
responsiveness, process-death recovery, physical pen feel, TalkBack speech, tablets/foldables,
and release signing/distribution remain separate validation and product work.

## ChromeOS / Googlebook desktop adaptation

The same Android app adapts to the window width: at 840 dp it shows a native side
panel with style, selection and viewport controls; smaller windows retain the compact
bottom controls. Touchscreen hardware is optional. This is compatibility implementation,
not certification on ChromeOS or Googlebook hardware.

Keyboard: Ctrl+Z / Ctrl+Shift+Z / Ctrl+Y undo/redo; Ctrl+S saves a copy; Ctrl+O opens;
Ctrl+N creates a new drawing through the dirty guard. P / V choose Pen / Select;
Delete removes selected ink; Escape cancels temporary input; + / − zoom; F fits the drawing.
Text fields retain their editing keys. Space-drag or middle-button drag pans; wheel pans,
Ctrl+wheel zooms around the pointer; secondary click opens native canvas actions and never
inks. The hover cursor follows the tool. Resize cancels temporary ink/scale while retaining
committed document/history/selection and preserving the viewport center.

Incoming content-URI VIEW intents and single-file drops use the same bounded Rust import
and Save/Discard/Cancel guard. Only grants offered by the sender are used. Drops keep their
transient permission until completion/cancellation; recreation retains it with the document.
No arbitrary filesystem paths or network URLs are accepted. Existing configuration retention
is not process-death recovery. Desktop replay extends the phone replay with actual key,
wheel, secondary-button, Space-drag, resize and incoming-file cancellation pathways.
Real ChromeOS/Googlebook hardware, external keyboard/trackpad and stylus remain to be tested.
