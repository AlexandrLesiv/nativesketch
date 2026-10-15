import Testing

@testable import NativeSketch

struct RenderScheduleStateTests {
  @Test func coalescingAndTwoFrameBound() {
    var state = RenderScheduleState()
    state.request()
    state.request()
    #expect(state.canSchedule)
    state.begin()
    state.submitted()
    #expect(!state.pending)
    state.request()
    state.begin()
    state.submitted()
    state.request()
    #expect(!state.canSchedule)
    state.completed()
    #expect(state.canSchedule)
    #expect(state.outstanding == 1)
  }

  @Test func unavailableDrawableKeepsPendingWithoutAllocatingFrame() {
    var state = RenderScheduleState()
    state.request()
    state.begin()
    state.deferDrawable()
    #expect(state.pending)
    #expect(state.outstanding == 0)
    state.request()
    #expect(state.canSchedule)
  }

  @Test func recoveryWaitsForOutstandingAndRetriesOnce() {
    var state = RenderScheduleState()
    state.request()
    state.begin()
    state.submitted()
    state.failed()
    #expect(!state.canSchedule)
    state.completed()
    #expect(state.canSchedule)
    state.beginRecovery()
    state.begin()
    state.failed()
    #expect(state.blocked)
    #expect(!state.canSchedule)
    #expect(!state.recoveryNeeded)
    // Only a later external invalidation permits another ordinary attempt.
    state.request()
    #expect(state.canSchedule)
    state.succeeded()
    state.failed()
    #expect(state.recoveryNeeded)
  }

  @Test func closeDoesNotResurrectPendingFrames() {
    var state = RenderScheduleState()
    state.request()
    state.begin()
    state.submitted()
    state.close()
    state.completed()
    state.request()
    #expect(!state.pending)
    #expect(!state.canSchedule)
    #expect(state.outstanding == 0)
  }
}
