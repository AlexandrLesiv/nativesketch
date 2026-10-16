import AppKit
import CSketchSession

@MainActor enum Integration {
  static func run() throws {
    let runtime = try NativeRuntime(services: false)
    let editor = try RustApplicationSession(runtime: runtime)
    let scene = SessionScene(session: editor)
    func stroke(_ pressure: Bool) {
      precondition(editor.send(.tool(SK_V1_PEN.rawValue)))
      precondition(
        editor.send(.brush(SketchBrushV1(red: 0.2, green: 0.3, blue: 0.7, alpha: 0.8, width: 10))))
      precondition(
        editor.send(.beginStroke(42, pressure ? SK_V1_STYLUS.rawValue : SK_V1_MOUSE.rawValue)))
      let samples = (0..<40).map { index in
        SketchSampleV1(
          x: Double(index) * 3, y: sin(Double(index) / 5) * 20,
          timestamp: Double(index) / 120, pressure: Double(index) / 39,
          sample_id: UInt64(index), has_pressure: pressure ? 1 : 0, kind: 0)
      }
      precondition(editor.append(samples, pointer: 42))
      precondition(editor.send(.commitStroke(42)))
    }
    func geometry() -> [Double] {
      scene.commands().flatMap {
        [$0.x1, $0.y1, $0.x2, $0.y2, $0.x3, $0.y3, $0.red, $0.green, $0.blue, $0.alpha]
      }
    }
    stroke(false)
    let mouse = geometry()
    precondition(editor.send(.undo))
    stroke(true)
    precondition(geometry() != mouse)
    let original = geometry()
    precondition(editor.send(.beginStroke(42, SK_V1_MOUSE.rawValue)))
    precondition(editor.send(.cancel) && geometry() == original)
    precondition(editor.send(.tool(SK_V1_SELECT.rawValue)))
    precondition(editor.send(.cycle(1)))
    precondition(editor.send(.beginTransform))
    precondition(editor.send(.transform(2, 0, 0)))
    precondition(editor.send(.cancel) && geometry() == original)
    precondition(editor.send(.beginTransform))
    precondition(editor.send(.transform(1.5, 0, 0)))
    precondition(editor.send(.commitTransform))
    let resized = geometry()
    precondition(resized != original)
    precondition(editor.send(.undo) && geometry() == original)
    precondition(editor.send(.redo) && geometry() == resized)
    try SmokeStorageJourney.run(editor: editor, expected: resized, geometry: geometry)
    try NativeRetirementSmoke.run()
    print(
      "PASS: versioned Swift/Rust dispatcher, mouse/pressure geometry, cancellation, transform history and async save/reopen."
    )
  }
}
