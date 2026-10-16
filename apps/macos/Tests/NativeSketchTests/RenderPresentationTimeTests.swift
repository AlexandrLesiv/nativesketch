import CSketchSession
import Testing

@testable import NativeSketch

struct RenderPresentationTimeTests {
  @Test func invalidMetalTimestampsCannotCertifyPresentation() {
    for time in [0, -1, 9, Double.infinity, -Double.infinity, Double.nan] {
      let observation = RenderPresentationTime(reported: time, submittedAt: 10)
      #expect(observation.acceptedTime == nil)
      #expect(observation.status != "presented")
      #expect(observation.reportedTime?.isFinite ?? true)
    }
  }

  @Test func validPresentationKeepsActualHostTimestamp() {
    for time in [10.0, 10.012] {
      let observation = RenderPresentationTime(reported: time, submittedAt: 10)
      #expect(observation.acceptedTime == time)
      #expect(observation.reportedTime == time)
      #expect(observation.status == "presented")
    }
  }

  @Test @MainActor func droppedEvidenceSurvivesAndDiagnosticsDoNotOwnAcceptance() {
    let diagnostics = RenderDiagnostics()
    let id = diagnostics.record(
      SketchRenderStatsV1(), session: 1, scene: 1, surface: 1,
      requestedAt: 9, submittedAt: 10, renderStartedAt: 9.5,
      acquisition: 0.01, outstanding: 1)
    diagnostics.presented(id, observation: RenderPresentationTime(reported: 0, submittedAt: 10))
    let records = diagnostics.snapshot()
    #expect(records[0].reportedPresentationTime == 0)
    #expect(records[0].presentationStatus == "not-presented-or-dropped")
    #expect(records[0].presentedAt == nil)
    #expect(RenderBenchmarkFrame(records[0]).requestToPresentationSeconds == nil)
    diagnostics.reset()
    let accepted = RenderPresentationTime(reported: 10.012, submittedAt: 10)
    diagnostics.presented(id, observation: accepted)
    #expect(accepted.acceptedTime == 10.012)
    #expect(diagnostics.snapshot().isEmpty)
  }

  @Test func presentationRetriesAreBoundedAcrossCompletionOrder() {
    for completionFirst in [true, false] {
      var state = RenderScheduleState()
      state.request()
      state.begin()
      state.submitted()
      if completionFirst { state.completed() }
      let first = state.reservePresentationRetry()
      #expect(first)
      if !completionFirst { state.completed() }
      state.deferDrawable()
      #expect(state.canSchedule)
      let second = state.reservePresentationRetry()
      let exhausted = state.reservePresentationRetry()
      #expect(second)
      #expect(!exhausted)
      state.request()
      let reset = state.reservePresentationRetry()
      #expect(reset)
      state.close()
      let closed = state.reservePresentationRetry()
      #expect(!closed)
    }
  }
}
