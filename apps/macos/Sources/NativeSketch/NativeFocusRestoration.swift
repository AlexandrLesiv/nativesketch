import AppKit

/// Native presentation state only; no document state or reusable field-editor ownership.
@MainActor final class NativeFocusRestoration: NSObject {
  unowned let owner: EditorWindowController
  private weak var origin: NSView?
  private var selection: NSRange?
  private var session: UInt64 = 0
  private var inspectorWasShown = false
  private var pending = false
  private var generation: UInt64 = 0
  init(owner: EditorWindowController) {
    self.owner = owner
    super.init()
    NotificationCenter.default.addObserver(
      self, selector: #selector(sheetEnded(_:)), name: NSWindow.didEndSheetNotification,
      object: owner.window)
  }
  deinit { NotificationCenter.default.removeObserver(self) }
  @discardableResult func capture() -> UInt64 {
    generation &+= 1
    pending = false
    session = owner.editor.projection.session
    inspectorWasShown = owner.inspectorController.inspector.isShown
    let responder = NSApp.keyWindow?.firstResponder
    origin = responder as? NSView
    selection = nil
    for field in [owner.inspectorController.width, owner.inspectorController.scale] {
      if let editor = field.currentEditor() as? NSTextView, editor === responder {
        origin = field
        selection = editor.selectedRange()
        break
      }
    }
    return generation
  }
  func captureForPresentation() -> UInt64 {
    if pending && session == owner.editor.projection.session {
      pending = false
      return generation
    }
    return capture()
  }
  func isCurrent(_ expected: UInt64) -> Bool {
    expected == generation && session == owner.editor.projection.session
  }
  func discard() {
    generation &+= 1
    origin = nil
    pending = false
  }
  func cancelled(session expected: UInt64, generation expectedGeneration: UInt64? = nil) {
    guard expected == session,
      expectedGeneration == nil || expectedGeneration == generation
    else { return }
    pending = true
    schedule()
  }
  @objc private func sheetEnded(_ notification: Notification) { schedule() }
  private func schedule() {
    guard pending else { return }
    let expected = generation
    let responder = owner.window.firstResponder
    Task { @MainActor [weak self, weak responder] in
      guard let self, expected == generation, pending,
        owner.window.attachedSheet == nil
      else { return }
      guard owner.window.firstResponder === responder else {
        discard()
        return
      }
      restore()
    }
  }
  private func restore() {
    defer { discard() }
    guard session == owner.editor.projection.session, NSApp.isActive,
      owner.window.isVisible,
      owner.window.isKeyWindow || origin?.window?.isKeyWindow == true,
      let origin
    else { return }
    if let control = origin as? NSControl, !control.isEnabled { return }
    let inspector = owner.inspectorController
    if inspectorWasShown && (origin === inspector.width || origin === inspector.scale) {
      inspector.ensureShown()
    }
    guard let window = origin.window, window.isVisible,
      window.makeFirstResponder(origin)
    else { return }
    if let field = origin as? NSTextField, let selection,
      let editor = field.currentEditor() as? NSTextView
    {
      let length = (editor.string as NSString).length
      let start = min(selection.location, length)
      editor.setSelectedRange(
        NSRange(location: start, length: min(selection.length, length - start)))
    }
  }
}
