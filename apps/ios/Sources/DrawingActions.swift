import CSketchFFI
import UIKit
import UniformTypeIdentifiers

extension DrawingController {
  @objc func changeTool() {
    canvas.cancelAll()
    if tool.selectedSegmentIndex == 0 { _ = editor.check(sketch_select_clear(editor.handle)) }
    refresh()
  }
  @objc func undoEdit() {
    canvas.cancelAll()
    _ = editor.check(sketch_undo(editor.handle))
    refresh()
  }
  @objc func redoEdit() {
    canvas.cancelAll()
    _ = editor.check(sketch_redo(editor.handle))
    refresh()
  }
  @objc func nextStroke() -> Bool {
    canvas.cancelAll()
    tool.selectedSegmentIndex = 1
    _ = editor.check(sketch_select_cycle(editor.handle, 1))
    refresh()
    return true
  }
  @objc func previousStroke() -> Bool {
    canvas.cancelAll()
    tool.selectedSegmentIndex = 1
    _ = editor.check(sketch_select_cycle(editor.handle, -1))
    refresh()
    return true
  }
  @objc func deleteStroke() {
    canvas.cancelAll()
    _ = editor.check(sketch_delete_selected(editor.handle))
    refresh()
  }
  @objc func escape() {
    canvas.cancelAll()
    if editor.handle != nil { _ = editor.check(sketch_select_clear(editor.handle)) }
    refresh()
  }
}
