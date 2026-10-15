import AppKit
import CSketchFFI

@MainActor final class EngineFailurePresenter: NSObject {
  unowned let owner: EditorWindowController
  init(owner: EditorWindowController) { self.owner = owner }
  var reportedEngineFailure = false
  func engineFailure(_ code: UInt32) {
    if code != 6 {
      owner.status.stringValue = "Engine operation failed (\(code))."
      return
    }
    guard !reportedEngineFailure else { return }
    reportedEngineFailure = true
    owner.canvas.input.gesture = .idle
    owner.refresh()
    let alert = NSAlert()
    alert.messageText = "The drawing engine stopped unexpectedly"
    alert.informativeText =
      "Unsaved changes are unavailable. Use New to start again or Open to reload a saved document. The last successfully saved file is unchanged."
    alert.runModal()
  }
  @discardableResult func check(_ code: UInt32) -> Bool {
    owner.editor.observe(code)
    if code == 6 || owner.editor.unavailable { return false }
    if code == 0 { return true }
    let a = NSAlert()
    a.messageText = "The operation could not be completed"
    a.informativeText = "Engine error \(code). Your previous committed document is retained."
    a.runModal()
    return false
  }
  func checkFile(_ code: UInt32, url: URL, verb: String) -> Bool {
    owner.editor.observe(code)
    if code == 6 || owner.editor.unavailable { return false }
    guard code != 0 else { return true }
    let alert = NSAlert()
    alert.messageText = "Could not \(verb) \(url.lastPathComponent)"
    let reason =
      code == 5
      ? "The file is not a supported Native Sketch document, or its contents are invalid."
      : "Check that the folder exists and that you have permission to access it. The disk may be full or unavailable."
    alert.informativeText = "\(reason)\n\(url.path)\nYour current document is retained."
    alert.runModal()
    return false
  }
}
