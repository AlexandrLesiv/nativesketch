import CSketchSession
import Foundation

/// Scheduling is derived from the latest lifecycle snapshot, never a previous poll output.
struct NativePumpDecision {
  let state: SketchRuntimeStateV1
  let retiring: Bool
  var delay: TimeInterval? {
    guard state.stopped == 0 else { return nil }
    if state.needs_poll != 0 || state.ready_events != 0 || retiring { return 0.016 }
    guard state.wake_after_ms != UInt64.max else { return nil }
    return max(0.016, Double(state.wake_after_ms) / 1000)
  }
}
