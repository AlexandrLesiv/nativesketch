import AppKit
import CSketchFFI

@main enum NativeSketch {
  @MainActor static func main() {
    guard sketch_abi_version() == 2 else { fatalError("Native engine ABI mismatch") }
    if CommandLine.arguments.contains("--smoke-test")
      || CommandLine.arguments.contains("--integration-test")
    {
      Integration.run()
      return
    }
    if CommandLine.arguments.contains("--timing") {
      Integration.timing()
      return
    }
    let app = NSApplication.shared
    app.setActivationPolicy(.regular)
    let delegate = AppDelegate()
    app.delegate = delegate
    withExtendedLifetime(delegate) { app.run() }
  }
}
