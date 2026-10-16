import AppKit
import CSketchSession

@MainActor final class EngineFailurePresenter {
  unowned let owner: EditorWindowController
  init(owner: EditorWindowController) { self.owner = owner }
  func engineFailure(_ code: UInt32) {
    if code == SK_V1_PANIC.rawValue || code == SK_V1_POISONED.rawValue {
      owner.canvas.input.gesture = .idle
      owner.status.stringValue =
        "The drawing engine is unavailable. Close this window; saved files are retained."
    } else {
      owner.status.stringValue =
        "The operation was not accepted (\(code)). Your document is retained."
    }
    NSAccessibility.post(
      element: owner.status, notification: .announcementRequested,
      userInfo: [
        .announcement: owner.status.stringValue,
        .priority: NSAccessibilityPriorityLevel.medium.rawValue,
      ])
  }
}
