import AppKit
import CSketchSession

@MainActor final class AppKitInputAdapter {
  unowned let canvas: CanvasView
  var owner: EditorWindowController { canvas.owner }
  var editor: RustApplicationSession { owner.editor }
  init(canvas: CanvasView) { self.canvas = canvas }
  var space = false
  enum Gesture { case idle, ink, resize, pan }
  var gesture = Gesture.idle
  var sampleID: UInt64 = 0
  var start = CGPoint.zero
  var anchor = CGPoint.zero
  var initialVector = CGPoint.zero
  func append(_ event: NSEvent) {
    canvas.renderDiagnostics.input(timestamp: event.timestamp)
    sampleID += 1
    let point = canvas.document(canvas.point(event))
    let pen = event.subtype == .tabletPoint || event.type == .tabletPoint
    let sample = SketchSampleV1(
      x: point.x, y: point.y, timestamp: event.timestamp,
      pressure: pen ? Double(event.pressure) : 0, sample_id: sampleID, has_pressure: pen ? 1 : 0,
      kind: 0)
    if !editor.append([sample], pointer: 1) { canvas.cancel() }
    canvas.needsDisplay = true
  }
  func mouseDown(with event: NSEvent) {
    guard !editor.unavailable, owner.inspectorController.fieldEditing.finishEditing(),
      canvas.window?.makeFirstResponder(canvas) == true
    else { return }
    start = canvas.point(event)
    if space {
      gesture = .pan
      return
    }
    if editor.projection.tool == SK_V1_PEN.rawValue {
      if editor.send(
        .beginStroke(
          1, event.subtype == .tabletPoint ? SK_V1_STYLUS.rawValue : SK_V1_MOUSE.rawValue))
      {
        gesture = .ink
        sampleID = 0
        append(event)
      }
    } else if let bounds = owner.scene.bounds(),
      let index = canvas.corners(bounds).firstIndex(where: {
        let point = canvas.screen($0)
        return hypot(point.x - start.x, point.y - start.y) <= 9
      })
    {
      let points = canvas.corners(bounds)
      anchor = points[(index + 2) % 4]
      initialVector = CGPoint(x: points[index].x - anchor.x, y: points[index].y - anchor.y)
      if editor.send(.beginTransform) { gesture = .resize }
    } else {
      let point = canvas.document(start)
      editor.send(.select(point.x, point.y, 6 / canvas.zoom))
    }
  }
  func mouseDragged(with event: NSEvent) {
    switch gesture {
    case .ink: append(event)
    case .resize:
      let point = canvas.document(canvas.point(event))
      let denominator = initialVector.x * initialVector.x + initialVector.y * initialVector.y
      let scale = max(
        0.01,
        ((point.x - anchor.x) * initialVector.x + (point.y - anchor.y) * initialVector.y)
          / max(denominator, 0.001))
      editor.send(.transform(scale, anchor.x, anchor.y))
    case .pan:
      let point = canvas.point(event)
      canvas.offset.x += point.x - start.x
      canvas.offset.y += point.y - start.y
      start = point
      canvas.needsDisplay = true
    case .idle: break
    }
  }
  func mouseUp(with event: NSEvent) {
    if gesture == .ink {
      append(event)
      if gesture == .ink { editor.send(.commitStroke(1)) }
    }
    if gesture == .resize { editor.send(.commitTransform) }
    gesture = .idle
    owner.runtime.pump.schedule()
    owner.refresh()
  }
}
