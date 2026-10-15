import AppKit

/// Platform replies only; Rust owns dirty/save/discard/navigation decisions.
@MainActor final class WindowLifecycleCoordinator {
  unowned let owner: EditorWindowController
  private var approved = false
  private var terminationPending = false
  init(owner: EditorWindowController) { self.owner = owner }

  func requestWindowClose() -> Bool {
    if approved { return true }
    _ = owner.documents.request(.close)
    return false
  }
  func requestTermination() -> NSApplication.TerminateReply {
    if approved && owner.runtime.handle == nil { return .terminateNow }
    if terminationPending { return .terminateLater }
    if approved {
      terminationPending = true
      return .terminateLater
    }
    guard owner.inspectorController.fieldEditing.finishEditing() else { return .terminateCancel }
    terminationPending = true
    guard owner.documents.request(.close) else {
      terminationPending = false
      return .terminateCancel
    }
    return .terminateLater
  }
  func closeConfirmed() {
    approved = true
    owner.window.close()
    owner.runtime.retire(owner.editor)
    owner.runtime.shutdown { [weak self] in
      guard let self else { return }
      if terminationPending {
        terminationPending = false
        NSApp.reply(toApplicationShouldTerminate: true)
      } else {
        NSApp.terminate(nil)
      }
    }
  }
  func navigationCancelled() {
    guard terminationPending else { return }
    terminationPending = false
    NSApp.reply(toApplicationShouldTerminate: false)
  }
}
