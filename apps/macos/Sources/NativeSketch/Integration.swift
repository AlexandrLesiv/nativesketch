import AppKit
import CSketchFFI

@MainActor enum Integration {
  static func run() {
    let e = Editor()
    func ok(_ status: UInt32) { precondition(status == 0, "FFI status \(status)") }
    func stroke(_ pressure: Bool) {
      ok(
        sketch_stroke_begin(
          e.handle, 42, pressure ? 1 : 0,
          SketchBrush(red: 0.2, green: 0.3, blue: 0.7, alpha: 0.8, width: 10)))
      var samples = (0..<40).map { i in
        SketchSample(
          x: Double(i) * 3, y: sin(Double(i) / 5) * 20, timestamp: Double(i) / 120,
          pressure: Double(i) / 39, sample_id: UInt64(i), has_pressure: pressure ? 1 : 0, kind: 0)
      }
      ok(
        samples.withUnsafeMutableBufferPointer {
          sketch_stroke_append(e.handle, 42, $0.baseAddress, $0.count)
        })
      ok(sketch_stroke_commit(e.handle, 42))
    }
    func geometry() -> [Double] {
      e.commands().flatMap {
        [
          Double($0.stroke_id), Double($0.verb), $0.red, $0.green, $0.blue, $0.alpha, $0.x1, $0.y1,
          $0.x2, $0.y2, $0.x3, $0.y3,
        ]
      }
    }
    stroke(false)
    let mouse = geometry()
    ok(sketch_undo(e.handle))
    stroke(true)
    precondition(geometry() != mouse)
    let original = geometry()
    precondition(e.state.stroke_count == 1)
    ok(
      sketch_stroke_begin(
        e.handle, 42, 0, SketchBrush(red: 0, green: 0, blue: 0, alpha: 1, width: 5)))
    ok(sketch_cancel(e.handle))
    precondition(geometry() == original)
    ok(sketch_select_cycle(e.handle, 1))
    ok(sketch_resize_begin(e.handle))
    ok(sketch_resize_update(e.handle, 2, 0, 0))
    ok(sketch_cancel(e.handle))
    precondition(geometry() == original)
    ok(sketch_resize_begin(e.handle))
    ok(sketch_resize_update(e.handle, 1.5, 0, 0))
    ok(sketch_resize_commit(e.handle))
    let resized = geometry()
    precondition(resized != original)
    ok(sketch_undo(e.handle))
    precondition(geometry() == original)
    ok(sketch_redo(e.handle))
    precondition(geometry() == resized)
    let directory = FileManager.default.temporaryDirectory.appendingPathComponent(
      "native-sketch-\(UUID().uuidString)")
    try! FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true)
    defer { try? FileManager.default.removeItem(at: directory) }
    let url = directory.appendingPathComponent("fixture.nsketch")
    ok(e.file(url, save: true))
    precondition(e.state.dirty == 0)
    ok(sketch_document_new(e.handle))
    ok(e.file(url, save: false))
    precondition(geometry() == resized && e.state.dirty == 0)
    stroke(false)
    let unsaved = geometry()
    precondition(
      e.file(directory.appendingPathComponent("missing/fixture.nsketch"), save: true) != 0
        && e.state.dirty != 0)
    let corrupt = directory.appendingPathComponent("corrupt.nsketch")
    try! Data("invalid document".utf8).write(to: corrupt)
    precondition(e.file(corrupt, save: false) != 0 && geometry() == unsaved)
    let fresh = Editor()
    ok(fresh.file(url, save: false))
    precondition(fresh.commands().count == resized.count / 12)
    print(
      "PASS: Swift/Rust integration — mouse/pressure geometry, cancellation, resize undo/redo, exact style+geometry save/reopen, save/open failure retention."
    )
  }
  static func timing() {
    let e = Editor()
    var samples: [SketchSample] = []
    for i in 0..<120 {
      let t = Double(i)
      let pressure = Double(i % 20) / 19.0
      samples.append(
        SketchSample(
          x: t * 2, y: sin(t / 8) * 30, timestamp: t / 120, pressure: pressure,
          sample_id: UInt64(i), has_pressure: 1, kind: 0))
    }
    let start = ProcessInfo.processInfo.systemUptime
    for _ in 0..<100 {
      _ = sketch_document_new(e.handle)
      _ = sketch_stroke_begin(
        e.handle, 1, 1, SketchBrush(red: 0, green: 0, blue: 0, alpha: 1, width: 8))
      samples.withUnsafeMutableBufferPointer {
        _ = sketch_stroke_append(e.handle, 1, $0.baseAddress, $0.count)
      }
      _ = sketch_stroke_commit(e.handle, 1)
      _ = e.commands()
    }
    print(
      "CPU replay+path copy: \((ProcessInfo.processInfo.systemUptime-start)*1000/100) ms/120-sample stroke; 100 iterations. No GPU/presentation or hardware-input latency claim."
    )
  }
}
