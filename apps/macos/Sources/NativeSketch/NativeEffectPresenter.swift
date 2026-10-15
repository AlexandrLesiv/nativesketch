import AppKit
import CSketchSession

@MainActor final class NativeEffectPresenter {
  unowned let owner: EditorWindowController
  private var token: SketchEffectTokenV1?
  private var sheet: NSWindow?
  private var failureOperation: UInt64?
  private var failureSession: UInt64?
  init(owner: EditorWindowController) { self.owner = owner }

  func chooseDestination(_ token: SketchEffectTokenV1) {
    guard begin(token) else { return }
    let panel = NSSavePanel()
    panel.nameFieldStringValue = owner.documents.url?.lastPathComponent ?? "Untitled.nsketch"
    sheet = panel
    panel.beginSheetModal(for: owner.window) { [weak self] result in
      guard let self, finish(token) else { return }
      if result == .OK, let url = panel.url {
        _ = owner.editor.supply(url, token: token)
        owner.refresh()
      } else {
        owner.editor.reply(token, SK_V1_REPLY_CANCEL)
      }
      restoreFocus(session: token.session)
    }
  }
  func chooseSource(_ token: SketchEffectTokenV1) {
    guard begin(token) else { return }
    let panel = NSOpenPanel()
    panel.allowsMultipleSelection = false
    sheet = panel
    panel.beginSheetModal(for: owner.window) { [weak self] result in
      guard let self, finish(token) else { return }
      if result == .OK, let url = panel.url {
        _ = owner.editor.supply(url, token: token)
        owner.refresh()
      } else {
        owner.editor.reply(token, SK_V1_REPLY_CANCEL)
      }
      restoreFocus(session: token.session)
    }
  }
  func confirmUnsaved(_ token: SketchEffectTokenV1) {
    guard begin(token) else { return }
    let alert = NSAlert()
    alert.messageText = "Save changes to \(owner.documents.displayName)?"
    alert.informativeText = "Your changes will be lost if you don’t save them."
    alert.addButton(withTitle: "Save")
    alert.addButton(withTitle: "Cancel")
    alert.addButton(withTitle: "Don’t Save")
    sheet = alert.window
    alert.beginSheetModal(for: owner.window) { [weak self] response in
      guard let self, finish(token) else { return }
      let reply =
        response == .alertFirstButtonReturn
        ? SK_V1_REPLY_SAVE
        : response == .alertThirdButtonReturn ? SK_V1_REPLY_DISCARD : SK_V1_REPLY_CANCEL
      owner.editor.reply(token, reply)
      restoreFocus(session: token.session)
    }
  }
  func failure(_ outcome: SketchOutcomeV1) {
    guard
      outcome.failure.session == 0 || outcome.failure.session == owner.editor.projection.session,
      outcome.acknowledgement != SK_V1_STALE_SESSION.rawValue,
      outcome.acknowledgement != SK_V1_STALE_BINDING.rawValue,
      sheet == nil, failureOperation != outcome.operation_id
    else { return }
    failureOperation = outcome.operation_id
    failureSession = owner.editor.projection.session
    let alert = NSAlert()
    let uncertain =
      outcome.save_result == SK_V1_UNCERTAIN.rawValue
      || outcome.durability == SK_V1_DIRECTORY_SYNC_UNCONFIRMED.rawValue
    alert.messageText =
      uncertain
      ? "The file was written, but durability is unconfirmed"
      : "The operation could not be completed"
    alert.informativeText =
      uncertain
      ? "Keep this document open and save again when storage is available."
      : "Your current document is retained. Check the destination and available storage."
    let retry = SessionCommand.retry(outcome.retry_action)
    if retry != nil { alert.addButton(withTitle: "Try Again") }
    alert.addButton(withTitle: "Save As…")
    alert.addButton(withTitle: "Cancel")
    sheet = alert.window
    let session = owner.editor.projection.session
    alert.beginSheetModal(for: owner.window) { [weak self] response in
      guard let self, failureSession == session else { return }
      sheet = nil
      failureSession = nil
      failureOperation = nil
      guard owner.editor.projection.session == session else { return }
      if let retry, response == .alertFirstButtonReturn {
        owner.documents.request(retry)
      } else if response.rawValue == NSApplication.ModalResponse.alertFirstButtonReturn.rawValue
        + (retry == nil ? 0 : 1)
      {
        owner.documents.request(.saveAs)
      }
      restoreFocus(session: session)
    }
  }
  func dismissIfObsolete(_ projection: SketchProjectionV1) {
    let obsolete =
      token.map { !matches($0, projection.continuation) }
      ?? (failureSession.map { $0 != projection.session } ?? false)
    guard obsolete, let old = sheet else { return }
    token = nil
    failureSession = nil
    sheet = nil
    owner.window.endSheet(old, returnCode: .cancel)
  }
  private func begin(_ next: SketchEffectTokenV1) -> Bool {
    guard sheet == nil else { return false }
    token = next
    return true
  }
  private func finish(_ expected: SketchEffectTokenV1) -> Bool {
    guard let token, matches(token, expected),
      matches(expected, owner.editor.projection.continuation)
    else { return false }
    self.token = nil
    sheet = nil
    return true
  }
  private func matches(_ lhs: SketchEffectTokenV1, _ rhs: SketchEffectTokenV1) -> Bool {
    lhs.owner == rhs.owner && lhs.session == rhs.session && lhs.serial == rhs.serial
      && lhs.request == rhs.request
  }
  private func restoreFocus(session: UInt64) {
    guard owner.editor.projection.session == session, owner.window.isVisible,
      owner.window.attachedSheet == nil, owner.actions.fieldEditor == nil
    else { return }
    owner.window.makeFirstResponder(owner.canvas)
  }
}
