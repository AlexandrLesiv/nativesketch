import CSketchFFI
import UIKit
import UniformTypeIdentifiers

extension DrawingView {
  func begin(at p: CGPoint, timestamp: Double, pressure: Double?, pencil: Bool) {
    guard editor.handle != nil else { return }
    becomeFirstResponder()
    if owner.tool.selectedSegmentIndex == 0 {
      var r: CGFloat = 0
      var g: CGFloat = 0
      var b: CGFloat = 0
      var a: CGFloat = 0
      owner.ink.getRed(&r, green: &g, blue: &b, alpha: &a)
      if editor.check(
        sketch_stroke_begin(
          editor.handle, 1, pencil ? 1 : 2,
          SketchBrush(red: r, green: g, blue: b, alpha: a, width: owner.strokeWidth)))
      {
        mode = 1
        sampleID = 0
        lastTimestamp = -Double.infinity
        move(to: p, timestamp: timestamp, pressure: pressure)
      }
    } else {
      let state = editor.state
      if state.selected_id != 0,
        let index = corners(state.selection).firstIndex(where: {
          let s = screen($0)
          return hypot(s.x - p.x, s.y - p.y) <= 22
        })
      {
        let points = corners(state.selection)
        anchor = points[(index + 2) % 4]
        initialVector = CGPoint(x: points[index].x - anchor.x, y: points[index].y - anchor.y)
        if editor.check(sketch_resize_begin(editor.handle)) { mode = 2 }
      } else {
        let p = document(p)
        _ = editor.check(sketch_select_at(editor.handle, p.x, p.y, 12 / zoom))
      }
    }
    owner.refresh()
  }
  func move(to p: CGPoint, timestamp: Double, pressure: Double?) {
    let d = document(p)
    if mode == 1 {
      guard timestamp > lastTimestamp else { return }
      lastTimestamp = timestamp
      sampleID += 1
      var sample = SketchSample(
        x: d.x, y: d.y, timestamp: timestamp, pressure: pressure ?? 0, sample_id: sampleID,
        has_pressure: pressure == nil ? 0 : 1, kind: 0)
      if !editor.check(sketch_stroke_append(editor.handle, 1, &sample, 1)) { cancel() }
    } else if mode == 2 {
      let denominator = max(
        0.001, initialVector.x * initialVector.x + initialVector.y * initialVector.y)
      let scale = max(
        0.01,
        ((d.x - anchor.x) * initialVector.x + (d.y - anchor.y) * initialVector.y) / denominator)
      if !editor.check(sketch_resize_update(editor.handle, scale, anchor.x, anchor.y)) { cancel() }
    }
    setNeedsDisplay()
  }
  func end() {
    if mode == 1 { _ = editor.check(sketch_stroke_commit(editor.handle, 1)) }
    if mode == 2 { _ = editor.check(sketch_resize_commit(editor.handle)) }
    mode = 0
    ownerTouch = nil
    owner.refresh()
  }
}
