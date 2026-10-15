import CSketchSession
import Foundation

/// Headless integration only: semantic replies replace UI; production storage paths stay real.
@MainActor enum SmokeStorageJourney {
  static func wait(
    _ runtime: NativeRuntime, stage: String, editor: RustApplicationSession? = nil,
    until condition: () -> Bool
  ) throws {
    let deadline = Date().addingTimeInterval(15)
    while !condition() && Date() < deadline {
      runtime.pump.drain()
      RunLoop.main.run(until: Date().addingTimeInterval(0.005))
    }
    guard condition() else {
      var state = SketchRuntimeStateV1()
      let status = sketch_runtime_v1_state(runtime.handle, &state)
      let projection = editor?.projection ?? SketchProjectionV1()
      fputs(
        "Smoke timeout stage=\(stage) status=\(status) session=\(projection.session) revision=\(projection.revision) phase=\(projection.file_phase) preparing=\(projection.preparing_operations) saves=\(projection.outstanding_saves) dirty=\(projection.dirty) pending=\(state.pending_operations) ready=\(state.ready_events) retained=\(state.retained_events) stopping=\(state.stopping) stopped=\(state.stopped)\n",
        stderr)
      throw NativeBridgeError(operation: stage, status: SK_V1_BUSY.rawValue)
    }
  }
  static func run(editor: RustApplicationSession, expected: [Double], geometry: () -> [Double])
    throws
  {
    let directory = FileManager.default.temporaryDirectory.appendingPathComponent(
      "native-session-smoke-\(UUID().uuidString)")
    try FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true)
    defer {
      do { try FileManager.default.removeItem(at: directory) } catch {
        NativeDiagnostics.failure("smoke-cleanup", status: SK_V1_IO.rawValue)
      }
    }
    let url = directory.appendingPathComponent("fixture.nsketch")
    var latest = SketchOutcomeV1()
    editor.onOutcome = { outcome in
      latest = outcome
      if outcome.failure.domain != 0 {
        fputs(
          "Smoke failure effect=\(outcome.effect_kind) domain=\(outcome.failure.domain) code=\(outcome.failure.code) operation=\(outcome.failure.operation)\n",
          stderr)
      }
    }
    precondition(editor.send(.saveAs))
    precondition(editor.supply(url, token: latest.effect))
    try wait(editor.runtime, stage: "save-as", editor: editor) {
      editor.projection.dirty == 0 && editor.projection.outstanding_saves == 0
    }
    precondition(FileManager.default.fileExists(atPath: url.path))
    let saved = try Data(contentsOf: url)
    precondition(editor.send(.new))
    try wait(editor.runtime, stage: "new", editor: editor) {
      editor.projection.stroke_count == 0
        && editor.projection.file_phase == SK_V1_FILE_IDLE.rawValue
    }
    precondition(editor.send(.open))
    precondition(editor.supply(url, token: latest.effect))
    try wait(editor.runtime, stage: "open", editor: editor) {
      editor.projection.stroke_count == 1
        && editor.projection.file_phase == SK_V1_FILE_IDLE.rawValue
    }
    precondition(geometry() == expected && editor.projection.dirty == 0)
    let corrupt = directory.appendingPathComponent("corrupt.nsketch")
    try Data("invalid document".utf8).write(to: corrupt)
    precondition(editor.send(.open))
    precondition(editor.supply(corrupt, token: latest.effect))
    try wait(editor.runtime, stage: "corrupt-open", editor: editor) {
      latest.effect_kind == SK_V1_FAILURE.rawValue
    }
    precondition(geometry() == expected && (try? Data(contentsOf: url)) == saved)
    precondition(editor.send(.close))
    precondition(editor.reply(latest.effect, SK_V1_REPLY_CONFIRM_CLOSE))
    editor.runtime.retire(editor)
    var stopped = false
    editor.runtime.shutdown { stopped = true }
    try wait(editor.runtime, stage: "shutdown", editor: editor) { stopped }
    editor.onOutcome = nil
  }
}
