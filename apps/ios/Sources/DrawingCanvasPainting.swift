import CSketchFFI
import UIKit
import UniformTypeIdentifiers

extension DrawingView {
  func corners(_ b: SketchBounds) -> [CGPoint] {
    [
      CGPoint(x: b.x, y: b.y), CGPoint(x: b.x + b.width, y: b.y),
      CGPoint(x: b.x + b.width, y: b.y + b.height), CGPoint(x: b.x, y: b.y + b.height),
    ]
  }
  func paint() {
    guard let c = UIGraphicsGetCurrentContext() else { return }
    c.setFillColor(UIColor.white.cgColor)
    c.fill(bounds)
    c.saveGState()
    c.translateBy(x: offset.x, y: offset.y)
    c.scaleBy(x: zoom, y: zoom)
    var id: UInt64?
    var path = CGMutablePath()
    var color = UIColor.black.cgColor
    func flush() {
      c.addPath(path)
      c.setFillColor(color)
      c.fillPath()
    }
    for command in editor.commands() {
      if id != command.stroke_id {
        if id != nil { flush() }
        id = command.stroke_id
        path = CGMutablePath()
        color =
          UIColor(red: command.red, green: command.green, blue: command.blue, alpha: command.alpha)
          .cgColor
      }
      switch command.verb {
      case 0: path.move(to: CGPoint(x: command.x1, y: command.y1))
      case 1: path.addLine(to: CGPoint(x: command.x1, y: command.y1))
      case 2:
        path.addCurve(
          to: CGPoint(x: command.x3, y: command.y3),
          control1: CGPoint(x: command.x1, y: command.y1),
          control2: CGPoint(x: command.x2, y: command.y2))
      default: path.closeSubpath()
      }
    }
    if id != nil { flush() }
    c.restoreGState()
    let s = editor.state
    if s.selected_id != 0 {
      let p = screen(CGPoint(x: s.selection.x, y: s.selection.y))
      c.setStrokeColor(UIColor.systemBlue.cgColor)
      c.setLineWidth(2)
      c.stroke(
        CGRect(x: p.x, y: p.y, width: s.selection.width * zoom, height: s.selection.height * zoom))
      for point in corners(s.selection) {
        let p = screen(point)
        let r = CGRect(x: p.x - 6, y: p.y - 6, width: 12, height: 12)
        c.setFillColor(UIColor.white.cgColor)
        c.fillEllipse(in: r)
        c.strokeEllipse(in: r)
      }
    }
  }
}
