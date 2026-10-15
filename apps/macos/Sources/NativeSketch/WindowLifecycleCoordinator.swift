import AppKit

/// Platform replies only; Rust owns dirty/save/discard/navigation decisions.
@MainActor final class WindowLifecycleCoordinator {
  unowned let owner: EditorWindowController
  private var approved = false
  private var terminationPending = false
  private var poisonWarning: NSAlert?
  init(owner: EditorWindowController) { self.owner = owner }

  func requestWindowClose() -> Bool {
    if approved { return true }
    if owner.editor.unavailable {
      confirmUnavailableClose()
      return false
    }
    _ = owner.documents.request(.close)
    return false
  }
  func requestTermination() -> NSApplication.TerminateReply {
    if approved && owner.runtime.handle == nil { return .terminateNow }
    if terminationPending { return .terminateLater }
    if approved {
      terminationPending = true
      beginShutdown()
      return .terminateLater
    }
    if owner.editor.unavailable {
      terminationPending = true
      confirmUnavailableClose()
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
    beginShutdown()
  }
  private func beginShutdown() {
    let accepted = owner.runtime.shutdown(
      onFailure: { [weak self] in
        self?.navigationCancelled()
      },
      { [weak self] in
        guard let self else { return }
        if terminationPending {
          terminationPending = false
          NSApp.reply(toApplicationShouldTerminate: true)
        } else {
          NSApp.terminate(nil)
        }
      })
    if !accepted { Task { @MainActor [weak self] in self?.navigationCancelled() } }
  }
  private func confirmUnavailableClose() {
    guard poisonWarning == nil else { return }
    if let sheet = owner.window.attachedSheet {
      owner.window.endSheet(sheet, returnCode: .cancel)
      Task { @MainActor [weak self] in self?.confirmUnavailableClose() }
      return
    }
    let alert = NSAlert()
    poisonWarning = alert
    alert.messageText = "Close the unavailable drawing engine?"
    alert.informativeText =
      "Unsaved changes cannot be recovered from this session. Previously accepted saves will finish before the app exits."
    alert.addButton(withTitle: "Cancel")
    alert.addButton(withTitle: "Close Window")
    alert.beginSheetModal(for: owner.window) { [weak self] response in
      guard let self else { return }
      poisonWarning = nil
      if response == .alertSecondButtonReturn { closeConfirmed() } else { navigationCancelled() }
    }
  }
  func navigationCancelled() {
    guard terminationPending else { return }
    terminationPending = false
    NSApp.reply(toApplicationShouldTerminate: false)
  }
}
