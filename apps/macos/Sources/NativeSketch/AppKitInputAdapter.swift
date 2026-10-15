import AppKit
import CSketchFFI

@MainActor final class AppKitInputAdapter {
  unowned let canvas: CanvasView
  var owner: EditorWindowController { canvas.owner }
  var editor: LegacyEditorBridge { owner.editor }
  init(canvas: CanvasView) { self.canvas = canvas }
  var space = false
  enum Gesture { case idle, ink, resize, pan }
  var gesture = Gesture.idle
  var sampleID: UInt64 = 0
  var start = CGPoint.zero
  var anchor = CGPoint.zero
  var initialVector = CGPoint.zero
  func append(_ event: NSEvent) {
    sampleID += 1
    let p = canvas.document(canvas.point(event))
    let pen = event.subtype == .tabletPoint || event.type == .tabletPoint
    var sample = SketchSample(
      x: p.x, y: p.y, timestamp: event.timestamp, pressure: pen ? Double(event.pressure) : 0,
      sample_id: sampleID, has_pressure: pen ? 1 : 0, kind: 0)
    let result = editor.observe(sketch_stroke_append(editor.handle, 1, &sample, 1))
    if result != 0 {
      canvas.cancel()
      owner.status.stringValue = "Input cancelled because the engine rejected a sample (\(result))."
    }
    canvas.needsDisplay = true
  }
  func mouseDown(with event: NSEvent) {
    guard !editor.unavailable else { return }
    canvas.window?.makeFirstResponder(canvas)
    start = canvas.point(event)
    if space {
      gesture = .pan
      return
    }
    if owner.inspectorController.tool.selectedSegment == 0 {
      let color = owner.inspectorController.color.color.usingColorSpace(.sRGB) ?? .black
      let brush = SketchBrush(
        red: color.redComponent, green: color.greenComponent, blue: color.blueComponent,
        alpha: color.alphaComponent, width: owner.inspectorController.width.doubleValue)
      if owner.failures.check(
        sketch_stroke_begin(editor.handle, 1, event.subtype == .tabletPoint ? 1 : 0, brush))
      {
        gesture = .ink
        sampleID = 0
        append(event)
      }
    } else {
      let state = editor.state
      if state.selected_id != 0,
        let index = canvas.corners(state.selection).firstIndex(where: {
          let p = canvas.screen($0)
          return hypot(p.x - start.x, p.y - start.y) <= 9
        })
      {
        let points = canvas.corners(state.selection)
        anchor = points[(index + 2) % 4]
        let p = points[index]
        initialVector = CGPoint(x: p.x - anchor.x, y: p.y - anchor.y)
        if owner.failures.check(sketch_resize_begin(editor.handle)) { gesture = .resize }
      } else {
        let p = canvas.document(start)
        _ = owner.failures.check(sketch_select_at(editor.handle, p.x, p.y, 6 / canvas.zoom))
      }
    }
    owner.refresh()
  }
  func mouseDragged(with event: NSEvent) {
    switch gesture {
    case .ink: append(event)
    case .resize:
      let p = canvas.document(canvas.point(event))
      let denominator = initialVector.x * initialVector.x + initialVector.y * initialVector.y
      let scale = max(
        0.01,
        ((p.x - anchor.x) * initialVector.x + (p.y - anchor.y) * initialVector.y)
          / max(denominator, 0.001))
      _ = owner.failures.check(sketch_resize_update(editor.handle, scale, anchor.x, anchor.y))
      canvas.needsDisplay = true
    case .pan:
      let p = canvas.point(event)
      canvas.offset.x += p.x - start.x
      canvas.offset.y += p.y - start.y
      start = p
      canvas.needsDisplay = true
    default: break
    }
  }
  func mouseUp(with event: NSEvent) {
    if gesture == .ink {
      append(event)
      if gesture == .ink { _ = owner.failures.check(sketch_stroke_commit(editor.handle, 1)) }
    }
    if gesture == .resize { _ = owner.failures.check(sketch_resize_commit(editor.handle)) }
    gesture = .idle
    owner.refresh()
  }
}
