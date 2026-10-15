import AppKit
import CSketchSession

@MainActor final class NativeEffectPresenter: NSObject {
  unowned let owner: EditorWindowController
  private var token: SketchEffectTokenV1?
  private var sheet: NSWindow?
  private var failureOperation: UInt64?
  private var failureSession: UInt64?
  private var pendingFailure: NativePendingFailure?
  private var observingSheetEnd = false
  init(owner: EditorWindowController) {
    self.owner = owner
    super.init()
  }
  deinit { NotificationCenter.default.removeObserver(self) }
  private func observeSheetEnd() {
    guard !observingSheetEnd, let window = owner.window else { return }
    observingSheetEnd = true
    NotificationCenter.default.addObserver(
      self, selector: #selector(sheetEnded(_:)), name: NSWindow.didEndSheetNotification,
      object: window)
  }
  @objc private func sheetEnded(_ notification: Notification) { schedulePendingFailure() }

  func chooseDestination(_ token: SketchEffectTokenV1) {
    guard begin(token) else { return }
    let panel = NSSavePanel()
    panel.nameFieldStringValue = owner.documents.url?.lastPathComponent ?? "Untitled.nsketch"
    sheet = panel
    panel.beginSheetModal(for: owner.window) { [weak self] result in
      guard let self, finish(token) else { return }
      if result == .OK, let url = panel.url {
        if !owner.editor.supply(url, token: token) {
          owner.editor.reply(token, SK_V1_REPLY_CANCEL)
        }
        owner.refresh()
      } else {
        owner.editor.reply(token, SK_V1_REPLY_CANCEL)
      }
      if result != .OK { owner.focusRestoration.cancelled(session: token.session) }
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
        if !owner.editor.supply(url, token: token) {
          owner.editor.reply(token, SK_V1_REPLY_CANCEL)
        }
        owner.refresh()
      } else {
        owner.editor.reply(token, SK_V1_REPLY_CANCEL)
      }
      if result != .OK { owner.focusRestoration.cancelled(session: token.session) }
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
      if reply == SK_V1_REPLY_CANCEL { owner.focusRestoration.cancelled(session: token.session) }
    }
  }
  func awaitWrites(_ token: SketchEffectTokenV1) {
    guard begin(token) else { return }
    let panel = NativeWaitingPanel(
      navigation: owner.editor.projection.navigation,
      terminating: owner.lifecycle.isTerminationPending)
    sheet = panel
    owner.window.beginSheet(panel) { [weak self] _ in
      guard let self, finish(token) else { return }
      owner.editor.reply(token, SK_V1_REPLY_CANCEL)
      owner.focusRestoration.cancelled(session: token.session)
    }
  }
  func failure(_ outcome: SketchOutcomeV1) {
    observeSheetEnd()
    NativeDiagnostics.failure(
      "file-result", status: outcome.failure.code,
      session: outcome.failure.session)
    var pending = NativePendingFailure(outcome: outcome)
    guard pending.isCurrent(owner.editor.projection) else { return }
    if failureOperation == outcome.operation_id
      || pendingFailure?.outcome.operation_id == outcome.operation_id
    {
      return
    }
    pending.additionalFailures = pendingFailure.map { $0.additionalFailures + 1 } ?? 0
    pendingFailure = pending
    presentPendingFailure()
  }
  func presentPendingFailure() {
    guard !owner.editor.unavailable, sheet == nil, owner.window.attachedSheet == nil,
      let pending = pendingFailure
    else {
      return
    }
    pendingFailure = nil
    guard pending.isCurrent(owner.editor.projection) else { return }
    let outcome = pending.outcome
    failureOperation = outcome.operation_id
    failureSession = owner.editor.projection.session
    let alert = NativeFailurePresentation.alert(
      outcome, additionalFailures: pending.additionalFailures)
    let retry = SessionCommand.retry(outcome.retry_action)
    sheet = alert.window
    let session = owner.editor.projection.session
    let focusGeneration = owner.focusRestoration.captureForPresentation()
    alert.beginSheetModal(for: owner.window) { [weak self] response in
      guard let self, failureSession == session else { return }
      sheet = nil
      failureSession = nil
      failureOperation = nil
      guard owner.editor.projection.session == session else { return }
      if let retry, response == .alertFirstButtonReturn {
        owner.documents.request(retry, preservingFocus: focusGeneration)
      } else if response.rawValue == NSApplication.ModalResponse.alertFirstButtonReturn.rawValue
        + (retry == nil ? 0 : 1)
      {
        owner.documents.request(.saveAs, preservingFocus: focusGeneration)
      } else {
        owner.focusRestoration.cancelled(session: session, generation: focusGeneration)
      }
      schedulePendingFailure()
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
    schedulePendingFailure()
  }
  private func begin(_ next: SketchEffectTokenV1) -> Bool {
    observeSheetEnd()
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
    schedulePendingFailure()
    return true
  }
  private func schedulePendingFailure() {
    Task { @MainActor [weak self] in self?.presentPendingFailure() }
  }
  private func matches(_ lhs: SketchEffectTokenV1, _ rhs: SketchEffectTokenV1) -> Bool {
    lhs.owner == rhs.owner && lhs.session == rhs.session && lhs.serial == rhs.serial
      && lhs.request == rhs.request
  }
}
