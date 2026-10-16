import CSketchSession
import Foundation
import Testing

@testable import NativeSketch

/// UNDO-001/002, CANCEL-001: real Swift/FFI/Rust commands, no windows or synthetic input.
@MainActor struct SessionActionTests {
  @Test func unavailableUndoPreservesFirstDraft() throws {
    try withSession { editor in
      begin(editor)
      let interaction = editor.projection.interaction
      #expect(!editor.available(SessionAction.undo))
      #expect(!editor.perform(.undo))
      #expect(editor.projection.interaction == interaction)
      #expect(editor.send(.commitStroke(1)))
      #expect(editor.projection.stroke_count == 1)
    }
  }

  @Test func compoundUndoCancelsDraftAndUndoesCommittedHistory() throws {
    try withSession { editor in
      begin(editor)
      #expect(editor.send(.commitStroke(1)))
      begin(editor)
      #expect(!editor.available(SK_V1_UNDO.rawValue))
      #expect(editor.available(SessionAction.undo))
      #expect(editor.perform(.undo))
      #expect(editor.projection.interaction == 0)
      #expect(editor.projection.stroke_count == 0)
      #expect(editor.perform(.redo))
      #expect(editor.projection.stroke_count == 1)
    }
  }

  @Test func invalidScalePreservesPreviewThenValidScaleCommitsOnce() throws {
    try withSession { editor in
      begin(editor)
      #expect(editor.send(.commitStroke(1)))
      #expect(editor.perform(.cycleSelection(.next)))
      #expect(editor.send(.beginTransform))
      #expect(editor.send(.transform(2, 0, 0)))
      let interaction = editor.projection.interaction
      let before = SessionScene(session: editor).bounds()
      #expect(!editor.perform(.scaleSelection(.nan)))
      #expect(editor.projection.interaction == interaction)
      let after = SessionScene(session: editor).bounds()
      #expect(before?.width == after?.width)
      #expect(editor.perform(.scaleSelection(3)))
      #expect(editor.projection.interaction == 0)
      #expect(editor.perform(.undo))
      #expect(editor.projection.stroke_count == 1)
      #expect(editor.perform(.undo))
      #expect(editor.projection.stroke_count == 0)
    }
  }

  private func begin(_ editor: RustApplicationSession) {
    #expect(editor.send(.beginStroke(1, SK_V1_MOUSE.rawValue)))
    #expect(
      editor.append(
        [
          SketchSampleV1(
            x: 10, y: 10, timestamp: 0, pressure: 0, sample_id: 0, has_pressure: 0, kind: 0),
          SketchSampleV1(
            x: 40, y: 40, timestamp: 1, pressure: 0, sample_id: 1, has_pressure: 0, kind: 0),
        ], pointer: 1))
  }

  private func withSession(_ body: (RustApplicationSession) throws -> Void) throws {
    let runtime = try NativeRuntime(services: false)
    let editor = try RustApplicationSession(runtime: runtime)
    defer {
      _ = editor.send(.cancel)
      var latest = SketchOutcomeV1()
      editor.onOutcome = { latest = $0 }
      #expect(editor.send(.close))
      if latest.effect_kind == SK_V1_CONFIRM_UNSAVED.rawValue {
        #expect(editor.reply(latest.effect, SK_V1_REPLY_DISCARD))
      }
      #expect(editor.reply(latest.effect, SK_V1_REPLY_CONFIRM_CLOSE))
      editor.onOutcome = nil
      runtime.retire(editor)
      var stopped = false
      _ = runtime.shutdown { stopped = true }
      let deadline = Date(timeIntervalSinceNow: 3)
      while !stopped && Date() < deadline {
        _ = RunLoop.main.run(mode: .default, before: Date(timeIntervalSinceNow: 0.01))
      }
      #expect(stopped)
    }
    try body(editor)
  }
}
