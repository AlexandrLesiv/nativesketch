import AppKit
import CSketchFFI

@MainActor final class NativeSmokeJourney: NSObject {
  unowned let owner: EditorWindowController
  init(owner: EditorWindowController) { self.owner = owner }
  func run() {
    func event(_ type: NSEvent.EventType, _ x: Double, _ y: Double, _ time: Double) -> NSEvent {
      NSEvent.mouseEvent(
        with: type, location: owner.canvas.convert(CGPoint(x: x, y: y), to: nil), modifierFlags: [],
        timestamp: time, windowNumber: owner.window.windowNumber, context: nil, eventNumber: 1,
        clickCount: 1, pressure: 0)!
    }
    owner.canvas.mouseDown(with: event(.leftMouseDown, 120, 120, 1))
    owner.canvas.mouseDragged(with: event(.leftMouseDragged, 180, 160, 2))
    owner.canvas.mouseUp(with: event(.leftMouseUp, 220, 180, 3))
    precondition(owner.editor.state.stroke_count == 1)
    owner.canvas.mouseDown(with: event(.leftMouseDown, 260, 220, 4))
    owner.canvas.cancel()
    owner.canvas.mouseUp(with: event(.leftMouseUp, 280, 240, 5))
    precondition(owner.editor.state.stroke_count == 1)
    owner.actions.nextStroke(nil)
    owner.inspectorController.scale.stringValue = "150"
    owner.actions.resize(nil)
    owner.actions.undo(nil)
    owner.actions.redo(nil)
    func geometry() -> [Double] {
      owner.editor.commands().flatMap {
        [$0.x1, $0.y1, $0.x2, $0.y2, $0.x3, $0.y3, $0.red, $0.green, $0.blue, $0.alpha]
      }
    }
    let before = geometry()
    owner.inspectorController.scale.stringValue = "200junk"
    owner.actions.resize(nil)
    precondition(geometry() == before)
    precondition(
      owner.status.stringValue == "Enter a scale greater than 0 and at most 10000 percent.")
    precondition(owner.inspectorController.scale.currentEditor() === owner.window.firstResponder)
    precondition(
      (owner.inspectorController.scale.currentEditor() as? NSTextView)?.selectedRange()
        == NSRange(location: 0, length: 7))
    owner.inspectorController.scale.stringValue = "100"
    let point = CGPoint(x: 42, y: 37)
    owner.canvas.setZoom(2)
    owner.window.setContentSize(NSSize(width: 700, height: 480))
    precondition(geometry() == before)
    let roundtrip = owner.canvas.document(owner.canvas.screen(point))
    precondition(abs(roundtrip.x - point.x) < 1e-10 && abs(roundtrip.y - point.y) < 1e-10)
    let saved = FileManager.default.temporaryDirectory.appendingPathComponent(
      "native-recovery-\(UUID().uuidString).nsketch")
    precondition(owner.editor.file(saved, save: true) == 0)
    let savedBytes = try! Data(contentsOf: saved)
    owner.documents.url = saved
    owner.editor.onFailure = nil  // Exercise recovery without opening an intentional panic alert.
    owner.editor.observe(6)
    owner.documents.recoverEditorIfNeeded()
    let missing = saved.appendingPathExtension("missing")
    precondition(owner.editor.file(missing, save: false) != 0)
    precondition(owner.documents.url == nil && owner.editor.state.stroke_count == 0)
    precondition((try! Data(contentsOf: saved)) == savedBytes)
    try? FileManager.default.removeItem(at: saved)
    print(
      "PASS: native AppKit mouse replay, cancellation, keyboard-equivalent selection/resize/history, viewport/window resize, failed recovery-open clears Save destination. Synthetic mouse events; no physical tablet."
    )
    _ = sketch_document_new(owner.editor.handle)
    NSApp.terminate(nil)
  }
}
