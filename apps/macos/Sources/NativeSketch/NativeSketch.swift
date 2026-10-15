import AppKit
import CSketchSession

@main enum NativeSketch {
  @MainActor static func main() {
    do {
      guard sketch_runtime_v1_version() == 1 else {
        throw NativeBridgeError(operation: "native-version", status: SK_V1_UNSUPPORTED.rawValue)
      }
      if CommandLine.arguments.contains("--smoke-test")
        || CommandLine.arguments.contains("--integration-test")
      {
        try Integration.run()
        return
      }
      if CommandLine.arguments.contains("--timing") {
        try NativeTiming.run()
        return
      }
      let runtime = try NativeRuntime(root: NativeLaunchConfiguration.runtimeRoot)
      try NativeLaunchConfiguration.configureTestControls(runtime)
      let app = NSApplication.shared
      app.setActivationPolicy(.regular)
      let delegate = try AppDelegate(runtime: runtime)
      app.delegate = delegate
      withExtendedLifetime(delegate) { app.run() }
    } catch {
      NativeDiagnostics.failure(
        "startup", status: (error as? NativeBridgeError)?.status ?? SK_V1_IO.rawValue)
      fputs("Native Sketch could not initialize its native runtime.\n", stderr)
      exit(1)
    }
  }
}
