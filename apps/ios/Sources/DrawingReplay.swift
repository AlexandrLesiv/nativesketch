import CSketchFFI
import UIKit
import UniformTypeIdentifiers

extension DrawingController {
  #if DEBUG
    func runReplay() {
      editor.failure = nil
      #if os(visionOS)
        precondition(tool.selectedSegmentIndex == 1)
        canvas.begin(at: CGPoint(x: 50, y: 50), timestamp: 0, pressure: nil, pencil: false)
        canvas.end()
        precondition(editor.state.stroke_count == 0)
        tool.selectedSegmentIndex = 0
      #endif
      canvas.begin(at: CGPoint(x: 50, y: 50), timestamp: 1, pressure: nil, pencil: false)
      canvas.move(to: CGPoint(x: 140, y: 110), timestamp: 2, pressure: nil)
      canvas.end()
      precondition(editor.state.stroke_count == 1)
      #if os(visionOS)
        canvas.begin(at: CGPoint(x: 180, y: 160), timestamp: 3, pressure: nil, pencil: false)
      #else
        canvas.begin(at: CGPoint(x: 180, y: 160), timestamp: 3, pressure: 0.5, pencil: true)
      #endif
      canvas.cancel()
      canvas.navigating = true
      precondition(canvas.navigating && canvas.mode == 0)
      canvas.cancelAll()
      canvas.end()
      precondition(editor.state.stroke_count == 1)
      _ = nextStroke()
      precondition(applyScale("150"))
      undoEdit()
      redoEdit()
      let fixture = FileManager.default.temporaryDirectory.appendingPathComponent(UUID().uuidString)
      try! Data(repeating: 42, count: 65).write(to: fixture)
      do {
        _ = try Self.readDocument(fixture, limit: 64)
        preconditionFailure("Oversize import accepted")
      } catch {}
      precondition((try! Self.readDocument(fixture, limit: 65)).count == 65)
      try? FileManager.default.removeItem(at: fixture)
      #if os(iOS)
        let beforeWatch = editor.snapshot()!.0
        var oversizedRejected = false
        watchSender.send(Data(repeating: 0, count: 8 * 1024 * 1024 + 1), title: "Fixture") {
          oversizedRejected = $0.contains("8 MiB")
        }
        precondition(
          oversizedRejected && editor.snapshot()!.0 == beforeWatch && editor.state.dirty != 0)
      #endif
      let beforeFit = editor.snapshot()!.0
      fitDrawing()
      precondition(editor.snapshot()!.0 == beforeFit)
      let snapshot = editor.snapshot()!
      precondition(editor.state.dirty != 0)
      precondition(!editor.load(Data("bad".utf8)))
      precondition(editor.state.stroke_count == 1)
      precondition(editor.check(sketch_document_mark_saved(editor.handle, snapshot.1)))
      precondition(editor.state.dirty == 0)
      precondition(editor.load(snapshot.0))
      precondition(editor.state.stroke_count == 1)
      let point = CGPoint(x: 45, y: 70)
      canvas.zoom = 2
      canvas.offset = CGPoint(x: 14, y: 34)
      let mapped = canvas.document(canvas.screen(point))
      precondition(abs(mapped.x - point.x) < 1e-9 && abs(mapped.y - point.y) < 1e-9)
      let previous = canvas.document(CGPoint(x: canvas.bounds.midX, y: canvas.bounds.midY))
      canvas.frame.size = CGSize(width: 320, height: 480)
      canvas.layoutSubviews()
      let center = canvas.document(CGPoint(x: canvas.bounds.midX, y: canvas.bounds.midY))
      precondition(abs(previous.x - center.x) < 1e-9 && abs(previous.y - center.y) < 1e-9)
      // Wide geometry exercises actual fit below the former 10% floor.
      precondition(editor.check(sketch_document_new(editor.handle)))
      tool.selectedSegmentIndex = 0
      canvas.zoom = 1
      canvas.offset = .zero
      canvas.begin(at: CGPoint(x: 0, y: 0), timestamp: 10, pressure: nil, pencil: false)
      canvas.move(to: CGPoint(x: 20000, y: 10000), timestamp: 11, pressure: nil)
      canvas.end()
      let wideBytes = editor.snapshot()!.0
      fitDrawing()
      precondition(canvas.zoom < 0.1)
      for p in [CGPoint(x: -3, y: -3), CGPoint(x: 20003, y: 10003)] {
        let projected = canvas.screen(p)
        precondition(projected.x >= 0 && projected.x <= canvas.bounds.width)
        precondition(projected.y >= 0 && projected.y <= canvas.bounds.height)
      }
      let fittedZoom = canvas.zoom
      zoomOut()
      precondition(canvas.zoom < fittedZoom)
      let gestureZoom = canvas.navigationZoom(canvas.zoom * 0.5)
      precondition(gestureZoom < canvas.zoom)
      precondition(editor.snapshot()!.0 == wideBytes)
      #if os(iOS)
        let blocker = UIAlertController(title: "Replay modal", message: nil, preferredStyle: .alert)
        blocker.addAction(UIAlertAction(title: "OK", style: .default))
        present(blocker, animated: false)
        precondition(presentedViewController === blocker)
        receiveWatchStatus("Watch transfer failed: replay fixture")
        precondition(lastWatchStatus == "Watch transfer failed: replay fixture")
        precondition(presentedViewController === blocker)
        dismiss(animated: false)
      #endif
      let text =
        "PASS UIKit adapter replay: ink, cancellation, selection, scale/history, invalid import retention, revision save acknowledgement, roundtrip, rotation center mapping, large-document fit/navigation and retained Watch status. Synthetic scalar input; no physical touch/Pencil."
      print(text)
      try? text.write(
        to: FileManager.default.urls(for: .documentDirectory, in: .userDomainMask)[0]
          .appendingPathComponent("replay-result.txt"), atomically: true, encoding: .utf8)
      exit(0)
    }
  #endif
}
