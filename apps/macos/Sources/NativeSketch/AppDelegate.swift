import AppKit

@MainActor final class AppDelegate: NSObject, NSApplicationDelegate {
  let runtime: NativeRuntime
  let controller: EditorWindowController
  init(runtime: NativeRuntime) throws {
    self.runtime = runtime
    controller = try EditorWindowController(runtime: runtime)
    super.init()
  }
  func applicationDidFinishLaunching(_ notification: Notification) { controller.show() }
  func applicationShouldTerminate(_ sender: NSApplication) -> NSApplication.TerminateReply {
    controller.lifecycle.requestTermination()
  }
  func applicationShouldTerminateAfterLastWindowClosed(_ sender: NSApplication) -> Bool { true }
  func applicationDidResignActive(_ notification: Notification) { controller.canvas?.cancel() }
}
