import AppKit
import CSketchSession

@MainActor final class DocumentCoordinator: NSObject, NSMenuItemValidation {
  unowned let owner: EditorWindowController
  lazy var sheets = NativeEffectPresenter(owner: owner)
  init(owner: EditorWindowController) { self.owner = owner }
  var url: URL? { owner.runtime.grants.url(owner.editor.projection.binding_grant) }
  var displayName: String { url?.lastPathComponent ?? "Untitled" }
  var busyMessage: String? {
    let state = owner.editor.projection
    if state.preparing_operations > 0 { return "Preparing…" }
    if state.file_phase == SK_V1_FILE_WRITES.rawValue { return "Finishing Save…" }
    if state.file_phase == SK_V1_FILE_REPLACEMENT.rawValue { return "Opening…" }
    return state.outstanding_saves > 0 ? "Saving…" : nil
  }

  @discardableResult func request(_ command: SessionCommand) -> Bool {
    owner.focusRestoration.capture()
    guard owner.inspectorController.fieldEditing.finishEditing() else {
      owner.focusRestoration.discard()
      return false
    }
    let accepted = owner.editor.send(command)
    if !accepted { owner.focusRestoration.discard() }
    if case .save = command, accepted, owner.editor.projection.continuation.serial == 0 {
      owner.focusRestoration.cancelled(session: owner.editor.projection.session)
    }
    return accepted
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
    case SK_V1_AWAIT_WRITES.rawValue: sheets.awaitWrites(outcome.effect)
    case SK_V1_CLOSE_READY.rawValue:
      owner.editor.reply(outcome.effect, SK_V1_REPLY_CONFIRM_CLOSE)
    case SK_V1_CLOSE_CONFIRMED.rawValue: owner.lifecycle.closeConfirmed()
    case SK_V1_CANCELLED.rawValue:
      owner.lifecycle.navigationCancelled()
    case SK_V1_FAILURE.rawValue:
      sheets.failure(outcome)
      owner.lifecycle.navigationCancelled()
    default: break
    }
    if outcome.has_save_result != 0 {
      if outcome.save_result == SK_V1_SAVE_FAILED.rawValue
        || outcome.save_result == SK_V1_UNCERTAIN.rawValue
        || outcome.save_result == SK_V1_FAILED_AFTER_CANCEL.rawValue
        || outcome.save_result == SK_V1_UNCERTAIN_AFTER_CANCEL.rawValue
        || outcome.durability == SK_V1_DIRECTORY_SYNC_UNCONFIRMED.rawValue
      {
        sheets.failure(outcome)
      }
    }
    if let busyMessage { owner.status.stringValue = busyMessage }
  }
  func validateMenuItem(_ item: NSMenuItem) -> Bool { owner.menus.validateMenuItem(item) }
}
