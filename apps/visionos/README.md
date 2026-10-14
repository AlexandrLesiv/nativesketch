# visionOS editor

The Apple Vision Pro host is a native UIKit 2D document window. It shares the
Swift adapter in `apps/ios/Sources` and the Rust ABI 2 engine. Standard navigation
bars, menus, segmented controls, alerts, Files and color pickers use the installed
system appearance. This is not an immersive or 3D drawing app.

Start in Select. Choose Pen explicitly to draw with a spatial pinch-and-drag;
choose Pan to navigate without modifying the document. Drawing options include
Zoom In/Out, Actual Size and Fit Drawing. The recognizer owns indirect input even
when UIKit exposes zero underlying touches; no synthetic pressure is supplied.
Temporary ink/resize is cancelled on loss of activation, tool changes or resize.

Build with Xcode and stable Rust visionOS targets installed:

```
rustup target add aarch64-apple-visionos aarch64-apple-visionos-sim
scripts/build-visionos simulator
scripts/run-visionos <booted-vision-simulator-UUID>
```

`device` builds are ad-hoc signed compile evidence; real installation requires
owner provisioning. Simulator builds include a `--integration-test` scalar adapter
replay; device builds do not. Replay is not evidence of real eye/hand targeting,
physical latency or pressure. Native recognizer behavior and Files integration
still require runtime inspection on a visionOS simulator/device.

The Core Graphics path painter is provisional; renderer selection remains open.

Fit Drawing supports valid documents that require scales below 10%; subsequent
zoom controls preserve those small scales. The adapter replay includes a
20,000-by-10,000-point drawing fitted into a 320-by-480-point canvas and checks
that fitting/navigation leave serialized document bytes unchanged.
