import CSketchSession
import Foundation

@MainActor enum RenderBenchmarkFixture {
  static let identity = "grid1000-pressure8-live120-v1"
  static func prepare(_ session: RustApplicationSession) throws {
    try require(
      session.send(.brush(SketchBrushV1(red: 0.12, green: 0.25, blue: 0.7, alpha: 0.7, width: 5))))
    for stroke in 0..<1000 {
      try require(session.send(.beginStroke(1, SK_V1_STYLUS.rawValue)))
      var samples: [SketchSampleV1] = []
      let baseX = Double(stroke % 40) * 20
      let baseY = Double(stroke / 40) * 18
      for index in 0..<8 {
        let x = baseX + Double(index) * 2
        let y = baseY + sin(Double(index)) * 3
        samples.append(sample(x: x, y: y, index: index, timestamp: Double(index) / 120))
      }
      try require(session.append(samples, pointer: 1))
      try require(session.send(.commitStroke(1)))
    }
  }
  static func step(_ phase: String, index: Int, owner: EditorWindowController) throws {
    let session = owner.editor
    switch phase {
    case "live":
      if index == 0 { try require(session.send(.beginStroke(1, SK_V1_STYLUS.rawValue))) }
      let time = ProcessInfo.processInfo.systemUptime
      owner.canvas.renderDiagnostics.input(timestamp: time)
      try require(
        session.append(
          [
            sample(
              x: 100 + Double(index) * 4, y: 250 + sin(Double(index) / 8) * 30,
              index: index, timestamp: time)
          ], pointer: 1))
    case "commit": try require(session.send(.commitStroke(1)))
    case "pan": owner.canvas.offset.x += 1
    default: break
    }
    owner.canvas.requestRepaint()
  }
  private static func sample(x: Double, y: Double, index: Int, timestamp: Double) -> SketchSampleV1
  {
    SketchSampleV1(
      x: x, y: y, timestamp: timestamp, pressure: 0.3 + Double(index % 8) / 12,
      sample_id: UInt64(index), has_pressure: 1, kind: 0)
  }
  private static func require(_ accepted: Bool) throws {
    if !accepted { throw RenderBenchmarkError(reason: "shared-command-rejected") }
  }
}

struct RenderBenchmarkError: Error { let reason: String }
