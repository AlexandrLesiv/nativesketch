import CSketchFFI
import UIKit
import UniformTypeIdentifiers

extension DrawingController {
  @objc func zoomIn() { setZoom(canvas.zoom * 1.25) }
  @objc func zoomOut() { setZoom(canvas.zoom / 1.25) }
  @objc func actualSize() { setZoom(1) }
  @objc func fitDrawing() {
    canvas.cancelAll()
    let path = CGMutablePath()
    for command in editor.commands() {
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
    guard !path.isEmpty else {
      actualSize()
      return
    }
    let b = path.boundingBoxOfPath
    canvas.zoom = min(
      16,
      min(
        max(1, canvas.bounds.width - 64) / max(1, b.width),
        max(1, canvas.bounds.height - 64) / max(1, b.height)))
    canvas.offset = CGPoint(
      x: canvas.bounds.midX - b.midX * canvas.zoom,
      y: canvas.bounds.midY - b.midY * canvas.zoom)
    refresh()
  }
  func setZoom(_ zoom: Double) {
    canvas.cancelAll()
    let p = CGPoint(x: canvas.bounds.midX, y: canvas.bounds.midY)
    let d = canvas.document(p)
    canvas.zoom = canvas.navigationZoom(zoom)
    canvas.offset = CGPoint(x: p.x - d.x * canvas.zoom, y: p.y - d.y * canvas.zoom)
    refresh()
  }
}
