import AppKit
import CSketchFFI

@MainActor final class EditorActions: NSObject, NSMenuItemValidation {
  unowned let owner: EditorWindowController
  init(owner: EditorWindowController) { self.owner = owner }
  @objc func penTool(_ sender: Any?) {
    owner.inspectorController.tool.selectedSegment = 0
    changeTool(sender)
    owner.window.makeFirstResponder(owner.canvas)
  }
  @objc func selectTool(_ sender: Any?) {
    owner.inspectorController.tool.selectedSegment = 1
    changeTool(sender)
    owner.window.makeFirstResponder(owner.canvas)
  }
  @objc func changeTool(_ sender: Any?) {
    owner.canvas.cancel()
    if owner.inspectorController.tool.selectedSegment == 0 {
      owner.editor.observe(sketch_select_clear(owner.editor.handle))
    }
    owner.refresh()
  }
  @objc func resize(_ sender: Any?) {
    owner.canvas.cancel()
    guard
      let percent = Double(
        owner.inspectorController.scale.stringValue.trimmingCharacters(in: .whitespacesAndNewlines)),
      percent.isFinite && percent > 0 && percent <= 10_000
    else {
      NSSound.beep()
      let message = "Enter a scale greater than 0 and at most 10000 percent."
      owner.window.makeFirstResponder(owner.inspectorController.scale)
      owner.inspectorController.scale.selectText(nil)
      owner.status.stringValue = message
      NSAccessibility.post(
        element: owner.inspectorController.scale, notification: .announcementRequested,
        userInfo: [.announcement: message, .priority: NSAccessibilityPriorityLevel.medium.rawValue])
      return
    }
    let factor = percent / 100
    guard factor.isFinite && factor > 0 && factor <= 100 else {
      NSSound.beep()
      return
    }
    let b = owner.editor.state.selection
    if owner.failures.check(sketch_resize_begin(owner.editor.handle))
      && owner.failures.check(
        sketch_resize_update(owner.editor.handle, factor, b.x + b.width / 2, b.y + b.height / 2))
    {
      _ = owner.failures.check(sketch_resize_commit(owner.editor.handle))
    }
    if owner.editor.state.interaction != 0 {
      if owner.editor.handle != nil { owner.editor.observe(sketch_cancel(owner.editor.handle)) }
    }
    owner.inspectorController.scale.stringValue = "100"
    owner.refresh()
  }
  @objc func nextStroke(_ sender: Any?) {
    owner.canvas.cancel()
    owner.inspectorController.tool.selectedSegment = 1
    _ = owner.failures.check(sketch_select_cycle(owner.editor.handle, 1))
    owner.refresh()
  }
  @objc func previousStroke(_ sender: Any?) {
    owner.canvas.cancel()
    owner.inspectorController.tool.selectedSegment = 1
    _ = owner.failures.check(sketch_select_cycle(owner.editor.handle, -1))
    owner.refresh()
  }
  @objc func deleteStroke(_ sender: Any?) {
    owner.canvas.cancel()
    _ = owner.failures.check(sketch_delete_selected(owner.editor.handle))
    owner.refresh()
  }
  @objc func undo(_ sender: Any?) {
    if let text = owner.window.firstResponder as? NSTextView {
      text.undoManager?.undo()
      return
    }
    owner.canvas.cancel()
    _ = owner.failures.check(sketch_undo(owner.editor.handle))
    owner.refresh()
  }
  @objc func redo(_ sender: Any?) {
    if let text = owner.window.firstResponder as? NSTextView {
      text.undoManager?.redo()
      return
    }
    owner.canvas.cancel()
    _ = owner.failures.check(sketch_redo(owner.editor.handle))
    owner.refresh()
  }
  @objc func zoomIn(_ sender: Any?) { owner.canvas.setZoom(owner.canvas.zoom * 1.25) }
  @objc func zoomOut(_ sender: Any?) { owner.canvas.setZoom(owner.canvas.zoom / 1.25) }
  @objc func actualSize(_ sender: Any?) { owner.canvas.setZoom(1) }
  func validateMenuItem(_ item: NSMenuItem) -> Bool { owner.menus.validateMenuItem(item) }
}
