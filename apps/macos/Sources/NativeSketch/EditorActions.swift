import AppKit
import CSketchSession

@MainActor final class EditorActions: NSObject, NSMenuItemValidation {
  unowned let owner: EditorWindowController
  init(owner: EditorWindowController) { self.owner = owner }
  var fieldEditor: NSTextView? {
    guard let text = NSApp.keyWindow?.firstResponder as? NSTextView, text.isFieldEditor else {
      return nil
    }
    return text
  }
  @objc func penTool(_ sender: Any?) { chooseTool(SK_V1_PEN.rawValue) }
  @objc func selectTool(_ sender: Any?) { chooseTool(SK_V1_SELECT.rawValue) }
  @objc func changeTool(_ sender: Any?) {
    chooseTool(
      owner.inspectorController.tool.selectedSegment == 0
        ? SK_V1_PEN.rawValue : SK_V1_SELECT.rawValue)
  }
  private func chooseTool(_ tool: UInt32) {
    guard owner.inspectorController.fieldEditing.finishEditing() else {
      owner.refresh()
      return
    }
    if perform(.chooseTool(tool)) { owner.window.makeFirstResponder(owner.canvas) }
  }
  @objc func resize(_ sender: Any?) {
    guard owner.inspectorController.fieldEditing.finishEditing() else { return }
    guard
      let percent = Double(
        owner.inspectorController.scale.stringValue.trimmingCharacters(in: .whitespacesAndNewlines)),
      percent.isFinite, percent > 0, percent <= 10_000
    else {
      NSSound.beep()
      let message = "Enter a scale greater than 0 and at most 10000 percent."
      owner.inspectorController.scale.window?.makeFirstResponder(owner.inspectorController.scale)
      owner.inspectorController.scale.selectText(nil)
      owner.status.stringValue = message
      NSAccessibility.post(
        element: owner.inspectorController.scale, notification: .announcementRequested,
        userInfo: [.announcement: message, .priority: NSAccessibilityPriorityLevel.medium.rawValue])
      return
    }
    if perform(.scaleSelection(percent / 100)) {
      owner.inspectorController.scale.stringValue = "100"
    }
  }
  @objc func nextStroke(_ sender: Any?) { cycle(.next) }
  @objc func previousStroke(_ sender: Any?) { cycle(.previous) }
  private func cycle(_ direction: SessionAction.CycleDirection) {
    guard owner.inspectorController.fieldEditing.finishEditing() else { return }
    perform(.cycleSelection(direction))
  }
  @objc func deleteStroke(_ sender: Any?) {
    guard fieldEditor == nil else { return }
    perform(.deleteSelection)
  }
  @objc func undo(_ sender: Any?) {
    if let fieldEditor {
      fieldEditor.undoManager?.undo()
      return
    }
    perform(.undo)
  }
  @objc func redo(_ sender: Any?) {
    if let fieldEditor {
      fieldEditor.undoManager?.redo()
      return
    }
    perform(.redo)
  }
  @discardableResult func perform(_ action: SessionAction) -> Bool {
    guard owner.editor.perform(action) else { return false }
    // Shared actions already cancelled Rust state. Never send a second Cancel.
    owner.canvas.input.gesture = .idle
    owner.canvas.input.space = false
    return true
  }
  @objc func zoomIn(_ sender: Any?) { owner.canvas.setZoom(owner.canvas.zoom * 1.25) }
  @objc func zoomOut(_ sender: Any?) { owner.canvas.setZoom(owner.canvas.zoom / 1.25) }
  @objc func actualSize(_ sender: Any?) { owner.canvas.setZoom(1) }
  func validateMenuItem(_ item: NSMenuItem) -> Bool { owner.menus.validateMenuItem(item) }
}
