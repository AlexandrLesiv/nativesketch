import AppKit
import CSketchSession

@MainActor final class NativeSmokeJourney {
  unowned let owner: EditorWindowController
  init(owner: EditorWindowController) { self.owner = owner }
  func run() {
    func event(_ type: NSEvent.EventType, _ x: Double, _ y: Double, _ time: Double) -> NSEvent {
      NSEvent.mouseEvent(
        with: type, location: owner.canvas.convert(CGPoint(x: x, y: y), to: nil),
        modifierFlags: [], timestamp: time, windowNumber: owner.window.windowNumber, context: nil,
        eventNumber: 1, clickCount: 1, pressure: 0)!
    }
    owner.canvas.mouseDown(with: event(.leftMouseDown, 120, 120, 1))
    owner.canvas.mouseDragged(with: event(.leftMouseDragged, 180, 160, 2))
    owner.canvas.mouseUp(with: event(.leftMouseUp, 220, 180, 3))
    precondition(owner.editor.projection.stroke_count == 1)
    owner.canvas.mouseDown(with: event(.leftMouseDown, 260, 220, 4))
    owner.canvas.cancel()
    owner.canvas.mouseUp(with: event(.leftMouseUp, 280, 240, 5))
    precondition(owner.editor.projection.stroke_count == 1)
    owner.actions.nextStroke(nil)
    owner.inspectorController.scale.stringValue = "150"
    owner.actions.resize(nil)
    owner.actions.undo(nil)
    owner.actions.redo(nil)
    let point = CGPoint(x: 42, y: 37)
    owner.canvas.setZoom(2)
    owner.window.setContentSize(NSSize(width: 700, height: 480))
    let roundtrip = owner.canvas.document(owner.canvas.screen(point))
    precondition(abs(roundtrip.x - point.x) < 1e-10 && abs(roundtrip.y - point.y) < 1e-10)
    print(
      "PASS: AppKit injected mouse replay through versioned shared dispatcher, cancel/selection/transform/history and viewport. Not native E2E or physical pen evidence."
    )
    owner.editor.onOutcome = { [weak owner] outcome in
      guard let owner else { return }
      switch outcome.effect_kind {
      case SK_V1_CONFIRM_UNSAVED.rawValue: owner.editor.reply(outcome.effect, SK_V1_REPLY_DISCARD)
      case SK_V1_CLOSE_READY.rawValue: owner.editor.reply(outcome.effect, SK_V1_REPLY_CONFIRM_CLOSE)
      case SK_V1_CLOSE_CONFIRMED.rawValue: owner.lifecycle.closeConfirmed()
      default: owner.refresh()
      }
    }
    owner.editor.send(.close)
  }
}
