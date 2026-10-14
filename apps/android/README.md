# Android native host

Kotlin Android Views and Material Components 1.13.0 controls, JNI ABI 2 transport,
and the shared Rust editor. Android Canvas paints Rust-generated paths provisionally;
this does not decide the production renderer. No network permission or runtime service.

Build with JDK 17+, Gradle 9.3.1, Android SDK 37 / build tools 36.0.0, NDK r30, and
Rust target `aarch64-linux-android`. Set `ANDROID_HOME`, `ANDROID_NDK_HOME`, `JAVA_HOME`,
and optionally `GRADLE` to local tool locations, then run `scripts/build-android`.
The resulting ARM64 debug APK is `artifacts/NativeSketch-android.apk`.
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
