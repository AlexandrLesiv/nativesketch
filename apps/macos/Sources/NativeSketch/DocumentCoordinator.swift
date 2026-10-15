import AppKit
import CSketchSession

@MainActor final class DocumentCoordinator: NSObject, NSMenuItemValidation {
  unowned let owner: EditorWindowController
  lazy var sheets = NativeEffectPresenter(owner: owner)
  init(owner: EditorWindowController) { self.owner = owner }
  var url: URL? { owner.runtime.grants.url(owner.editor.projection.binding_grant) }
  var displayName: String { url?.lastPathComponent ?? "Untitled" }
  var busyMessage: String?

  @discardableResult func request(_ command: SessionCommand) -> Bool {
    guard owner.inspectorController.fieldEditing.finishEditing() else { return false }
    guard owner.window.makeFirstResponder(owner.canvas) else { return false }
    return owner.editor.send(command)
  }
  @objc func saveDocument(_ sender: Any?) { request(.save) }
  @objc func saveAs(_ sender: Any?) { request(.saveAs) }
  @objc func newDocument(_ sender: Any?) { request(.new) }
  @objc func openDocument(_ sender: Any?) { request(.open) }
  @objc func showRecovery(_ sender: Any?) { owner.recoveryPresenter.show() }
  @objc func cancelNavigation(_ sender: Any?) {
    let token = owner.editor.projection.continuation
    guard token.serial != 0 else { return }
    owner.editor.reply(token, SK_V1_REPLY_CANCEL)
  }

  func receive(_ outcome: SketchOutcomeV1) {
    sheets.dismissIfObsolete(outcome.projection)
    switch outcome.effect_kind {
    case SK_V1_CHOOSE_DESTINATION.rawValue: sheets.chooseDestination(outcome.effect)
    case SK_V1_CHOOSE_SOURCE.rawValue: sheets.chooseSource(outcome.effect)
    case SK_V1_CONFIRM_UNSAVED.rawValue: sheets.confirmUnsaved(outcome.effect)
    case SK_V1_AWAIT_WRITES.rawValue: busyMessage = "Finishing Save…"
    case SK_V1_PREPARING.rawValue: busyMessage = "Preparing…"
    case SK_V1_OPENING.rawValue: busyMessage = "Opening…"
    case SK_V1_SAVE_QUEUED.rawValue: busyMessage = "Saving…"
    case SK_V1_CLOSE_READY.rawValue:
      owner.editor.reply(outcome.effect, SK_V1_REPLY_CONFIRM_CLOSE)
    case SK_V1_CLOSE_CONFIRMED.rawValue: owner.lifecycle.closeConfirmed()
    case SK_V1_CANCELLED.rawValue:
      busyMessage = nil
      owner.lifecycle.navigationCancelled()
    case SK_V1_FAILURE.rawValue:
      busyMessage = nil
      sheets.failure(outcome)
      owner.lifecycle.navigationCancelled()
    default: break
    }
    if outcome.has_save_result != 0 {
      if outcome.save_result == SK_V1_SAVE_FAILED.rawValue
        || outcome.save_result == SK_V1_UNCERTAIN.rawValue
      {
        sheets.failure(outcome)
      }
      if outcome.projection.outstanding_saves == 0 { busyMessage = nil }
    }
    if let busyMessage { owner.status.stringValue = busyMessage }
  }
  func validateMenuItem(_ item: NSMenuItem) -> Bool { owner.menus.validateMenuItem(item) }
}
