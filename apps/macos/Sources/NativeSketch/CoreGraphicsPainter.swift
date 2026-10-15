import AppKit
import CSketchFFI

// Provisional painter retained unchanged until task 020.
@MainActor final class CoreGraphicsPainter {
  unowned let canvas: CanvasView
  init(canvas: CanvasView) { self.canvas = canvas }
  func draw() {
    NSColor.white.setFill()
    canvas.bounds.fill()
    guard let context = NSGraphicsContext.current?.cgContext else { return }
    context.saveGState()
    context.translateBy(x: canvas.offset.x, y: canvas.offset.y)
    context.scaleBy(x: canvas.zoom, y: canvas.zoom)
    var id: UInt64?
    var path = CGMutablePath()
    var color = NSColor.black.cgColor
    func flush() {
      context.addPath(path)
      context.setFillColor(color)
      context.fillPath()
    }
    for c in canvas.editor.commands() {
      if id != c.stroke_id {
        if id != nil { flush() }
        path = CGMutablePath()
        id = c.stroke_id
        color = NSColor(srgbRed: c.red, green: c.green, blue: c.blue, alpha: c.alpha).cgColor
      }
      switch c.verb {
      case 0: path.move(to: CGPoint(x: c.x1, y: c.y1))
      case 1: path.addLine(to: CGPoint(x: c.x1, y: c.y1))
      case 2:
        path.addCurve(
          to: CGPoint(x: c.x3, y: c.y3), control1: CGPoint(x: c.x1, y: c.y1),
          control2: CGPoint(x: c.x2, y: c.y2))
      default: path.closeSubpath()
      }
    }
    if id != nil { flush() }
    context.restoreGState()
    if canvas.editor.state.selected_id != 0 {
      let b = canvas.editor.state.selection
      let p = canvas.screen(CGPoint(x: b.x, y: b.y))
      let r = CGRect(x: p.x, y: p.y, width: b.width * canvas.zoom, height: b.height * canvas.zoom)
      NSColor.systemBlue.setStroke()
      let outline = NSBezierPath(rect: r)
      outline.lineWidth = 1.5
      outline.stroke()
      for p in canvas.corners(b) {
        let s = canvas.screen(p)
        NSColor.white.setFill()
        let h = NSBezierPath(rect: CGRect(x: s.x - 4, y: s.y - 4, width: 8, height: 8))
        h.fill()
        NSColor.systemBlue.setStroke()
        h.stroke()
      }
    }
    if canvas.window?.firstResponder === canvas {
      NSColor.keyboardFocusIndicatorColor.setStroke()
      let p = NSBezierPath(rect: canvas.bounds.insetBy(dx: 2, dy: 2))
      p.lineWidth = 2
      p.stroke()
    }
  }
}
