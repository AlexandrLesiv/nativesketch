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
    owner.canvas.cancel()
    if owner.editor.send(.tool(tool)) { owner.window.makeFirstResponder(owner.canvas) }
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
    owner.canvas.cancel()
    guard let bounds = owner.scene.bounds(), owner.editor.send(.beginTransform) else { return }
    if owner.editor.send(
      .transform(percent / 100, bounds.x + bounds.width / 2, bounds.y + bounds.height / 2))
    {
      _ = owner.editor.send(.commitTransform)
    }
    if owner.editor.projection.interaction != 0 { owner.editor.send(.cancel) }
    owner.inspectorController.scale.stringValue = "100"
  }
  @objc func nextStroke(_ sender: Any?) { cycle(1) }
  @objc func previousStroke(_ sender: Any?) { cycle(-1) }
  private func cycle(_ direction: Int32) {
    guard owner.inspectorController.fieldEditing.finishEditing() else { return }
    owner.canvas.cancel()
    if owner.editor.send(.tool(SK_V1_SELECT.rawValue)) { owner.editor.send(.cycle(direction)) }
  }
  @objc func deleteStroke(_ sender: Any?) {
    guard fieldEditor == nil else { return }
    owner.canvas.cancel()
    owner.editor.send(.delete)
  }
  @objc func undo(_ sender: Any?) {
    if let fieldEditor {
      fieldEditor.undoManager?.undo()
      return
    }
    owner.canvas.cancel()
    owner.editor.send(.undo)
  }
  @objc func redo(_ sender: Any?) {
    if let fieldEditor {
      fieldEditor.undoManager?.redo()
      return
    }
    owner.canvas.cancel()
    owner.editor.send(.redo)
  }
  @objc func zoomIn(_ sender: Any?) { owner.canvas.setZoom(owner.canvas.zoom * 1.25) }
  @objc func zoomOut(_ sender: Any?) { owner.canvas.setZoom(owner.canvas.zoom / 1.25) }
  @objc func actualSize(_ sender: Any?) { owner.canvas.setZoom(1) }
  func validateMenuItem(_ item: NSMenuItem) -> Bool { owner.menus.validateMenuItem(item) }
}
