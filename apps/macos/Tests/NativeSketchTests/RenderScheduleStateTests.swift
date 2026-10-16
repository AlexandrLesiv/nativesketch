import Foundation
import Testing

@testable import NativeSketch

struct RenderScheduleStateTests {
  @Test func suspensionPreservesPendingWorkWithoutSchedulingFrames() {
    var state = RenderScheduleState()
    state.request()
    state.setSuspended(true)
    #expect(state.pending)
    #expect(!state.canSchedule)
    state.setSuspended(false)
    #expect(state.canSchedule)
    state.close()
    state.setSuspended(false)
    #expect(!state.canSchedule)
  }

  @Test func fractionalLogicalResizeAdvancesGenerationEvenWithSamePixelSize() {
    var surface = RenderSurfaceState()
    let initial = surface.update(CGSize(width: 100.1, height: 100), scale: 2)
    #expect(initial)
    let pixels = surface.pixels
    let generation = surface.generation
    let changed = surface.update(CGSize(width: 100.2, height: 100), scale: 2)
    #expect(changed)
    #expect(surface.pixels == pixels)
    #expect(surface.generation == generation + 1)
    let unchanged = surface.update(CGSize(width: 100.2, height: 100), scale: 2)
    #expect(!unchanged)
  }

  @Test func drawableRetriesAreBoundedAndCloseCancelsThem() {
    var state = RenderScheduleState()
    state.request()
    state.deferDrawable()
    let attempts = (0..<3).map { _ in state.reserveDrawableRetry() }
    #expect(attempts == [true, true, false])
    #expect(state.pending)
    state.request()
    let later = state.reserveDrawableRetry()
    #expect(later)
    state.close()
    let closed = state.reserveDrawableRetry()
    #expect(!closed)
  }

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
