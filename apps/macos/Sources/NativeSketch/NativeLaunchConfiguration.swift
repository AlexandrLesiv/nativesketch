import CSketchSession
import Foundation

#if NATIVE_TEST_CONTROLS
  import CSketchSessionTest
#endif

@MainActor enum NativeLaunchConfiguration {
  static func argument(_ name: String) -> String? {
    guard let index = CommandLine.arguments.firstIndex(of: name),
      CommandLine.arguments.indices.contains(index + 1)
    else { return nil }
    return CommandLine.arguments[index + 1]
  }
  static var runtimeRoot: URL? {
    #if DEBUG
      if let path = argument("--runtime-root") { return URL(fileURLWithPath: path) }
    #endif
    return nil
  }
  static func configureTestControls(_ runtime: NativeRuntime) throws {
    #if NATIVE_TEST_CONTROLS
      guard let root = argument("--test-storage-root"),
        let stageText = argument("--test-storage-stage"), let stage = UInt32(stageText),
        let modeText = argument("--test-storage-mode"), let mode = UInt32(modeText)
      else { return }
      let status = withNativeText(root) { text in
        var config = SketchTestControlV1(disposable_control_root: text, stage: stage, mode: mode)
        return sketch_runtime_v1_test_arm(runtime.handle, &config)
      }
      guard status == SK_V1_OK.rawValue else {
        throw NativeBridgeError(operation: "test-control-arm", status: status)
      }
    #endif
  }
}
