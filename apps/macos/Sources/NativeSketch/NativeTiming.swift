import CSketchSession
import Foundation

/// Historical CPU-only workload, migrated to the same typed session/scene consumer.
@MainActor enum NativeTiming {
  static func run() throws {
    let runtime = try NativeRuntime(services: false)
    let session = try RustApplicationSession(runtime: runtime)
    let scene = SessionScene(session: session)
    session.onOutcome = { [weak session] outcome in
      if outcome.effect_kind == SK_V1_CONFIRM_UNSAVED.rawValue {
        session?.reply(outcome.effect, SK_V1_REPLY_DISCARD)
      } else if outcome.effect_kind == SK_V1_CLOSE_READY.rawValue {
        session?.reply(outcome.effect, SK_V1_REPLY_CONFIRM_CLOSE)
      }
    }
    var appendTimes: [Double] = []
    var copyTimes: [Double] = []
    var rows = ["iteration,sample,input_append_us,path_copy_us"]
    for iteration in -10..<100 {
      precondition(session.send(.new))
      SmokeStorageJourney.wait(runtime) {
        session.projection.file_phase == SK_V1_FILE_IDLE.rawValue
          && session.projection.stroke_count == 0
      }
      precondition(
        session.send(.brush(SketchBrushV1(red: 0, green: 0, blue: 0, alpha: 1, width: 8))))
      precondition(session.send(.beginStroke(1, SK_V1_STYLUS.rawValue)))
      for index in 0..<120 {
        let t = Double(index)
        let sample = SketchSampleV1(
          x: t * 2, y: sin(t / 8) * 30, timestamp: t / 120,
          pressure: Double(index % 20) / 19, sample_id: UInt64(index), has_pressure: 1, kind: 0)
        let start = ProcessInfo.processInfo.systemUptime
        precondition(session.append([sample], pointer: 1))
        let appended = ProcessInfo.processInfo.systemUptime
        _ = scene.commands()
        let copied = ProcessInfo.processInfo.systemUptime
        if iteration >= 0 {
          let append = (appended - start) * 1_000_000
          let copy = (copied - appended) * 1_000_000
          appendTimes.append(append)
          copyTimes.append(copy)
          rows.append("\(iteration),\(index),\(append),\(copy)")
        }
      }
      precondition(session.send(.commitStroke(1)))
    }
    let destination =
      NativeLaunchConfiguration.argument("--timing-csv")
      ?? FileManager.default.temporaryDirectory.appendingPathComponent(
        "native-sketch-timing-\(UUID().uuidString).csv"
      ).path
    try rows.joined(separator: "\n").write(toFile: destination, atomically: true, encoding: .utf8)
    summary("input_append", appendTimes)
    summary("path_copy", copyTimes)
    print(
      "Raw CSV: \(destination). 10 warmup +100 measured120-sample strokes; typed bridge CPU workload only, not paint/GPU/presentation/physical latency."
    )
    precondition(session.send(.close))
    runtime.retire(session)
    var stopped = false
    runtime.shutdown { stopped = true }
    SmokeStorageJourney.wait(runtime) { stopped }
  }
  private static func summary(_ name: String, _ values: [Double]) {
    let sorted = values.sorted()
    func percentile(_ fraction: Double) -> Double {
      sorted[min(sorted.count - 1, Int(Double(sorted.count - 1) * fraction))]
    }
    print(
      "\(name) microseconds: p50=\(percentile(0.5)), p95=\(percentile(0.95)), p99=\(percentile(0.99))"
    )
  }
}
