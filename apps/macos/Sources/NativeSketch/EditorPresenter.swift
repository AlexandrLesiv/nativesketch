import AppKit
import CSketchSession

@MainActor final class EditorPresenter {
  unowned let owner: EditorWindowController
  init(owner: EditorWindowController) { self.owner = owner }

  func receive(_ outcome: SketchOutcomeV1) {
    if outcome.change_flags & SK_V1_CHANGED_SESSION.rawValue != 0 {
      owner.canvas.resetViewport()
      owner.editor.registerRecovery(name: owner.documents.displayName)
    }
    if outcome.change_flags != 0 || outcome.effect_kind != SK_V1_EFFECT_NONE.rawValue
      || outcome.has_save_result != 0
    {
      refresh()
    }
    if outcome.invalidation != SK_V1_INVALIDATE_NONE.rawValue { owner.canvas.needsDisplay = true }
    owner.documents.receive(outcome)
  }

  func refresh() {
    guard owner.canvas != nil else { return }
    let state = owner.editor.projection
    let inspector = owner.inspectorController
    owner.window.isDocumentEdited = state.dirty != 0
    owner.window.title = "\(owner.documents.displayName) — Native Sketch"
    inspector.tool.selectedSegment = state.tool == SK_V1_PEN.rawValue ? 0 : 1
    inspector.tool.setEnabled(owner.editor.available(SK_V1_CHOOSE_PEN.rawValue), forSegment: 0)
    inspector.tool.setEnabled(owner.editor.available(SK_V1_CHOOSE_SELECT.rawValue), forSegment: 1)
    let brushEnabled = owner.editor.available(SK_V1_SET_BRUSH.rawValue)
    for control in [inspector.color, inspector.width, inspector.stepper] as [NSControl] {
      control.isEnabled = brushEnabled
    }
    if inspector.width.currentEditor() == nil { inspector.width.doubleValue = state.brush.width }
    inspector.stepper.doubleValue = state.brush.width
    inspector.color.color = NSColor(
      srgbRed: state.brush.red, green: state.brush.green,
      blue: state.brush.blue, alpha: state.brush.alpha)
    let transformEnabled = owner.editor.available(SK_V1_BEGIN_TRANSFORM.rawValue)
    inspector.scale.isEnabled = transformEnabled
    inspector.apply.isEnabled = transformEnabled
    let selected = state.selected_id != 0
    let saving =
      state.preparing_operations > 0
      ? " · Preparing…" : state.outstanding_saves > 0 ? " · Saving…" : ""
    owner.status.stringValue =
      "\(state.stroke_count) stroke\(state.stroke_count == 1 ? "" : "s") · \(selected ? "Stroke selected" : state.tool == SK_V1_PEN.rawValue ? "Pen — drag to draw" : "Select — click a stroke") · \(Int(owner.canvas.zoom * 100))%\(saving)"
    owner.canvas.setAccessibilityValue(
      "\(state.stroke_count) strokes, \(selected ? "one selected" : "none selected"), zoom \(Int(owner.canvas.zoom * 100)) percent"
    )
    owner.canvas.needsDisplay = true
  }
}
