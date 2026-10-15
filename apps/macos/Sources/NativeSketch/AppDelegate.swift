import AppKit
import CSketchFFI

@MainActor final class AppDelegate: NSObject, NSApplicationDelegate {
  let controller = EditorWindowController()
  func applicationDidFinishLaunching(_ notification: Notification) { controller.show() }
  func applicationShouldTerminate(_ sender: NSApplication) -> NSApplication.TerminateReply {
    (controller.documents.approvedClose || controller.documents.mayReplace())
      ? .terminateNow : .terminateCancel
  }
  func applicationShouldTerminateAfterLastWindowClosed(_ sender: NSApplication) -> Bool { true }
  func applicationDidResignActive(_ notification: Notification) { controller.canvas?.cancel() }
}
