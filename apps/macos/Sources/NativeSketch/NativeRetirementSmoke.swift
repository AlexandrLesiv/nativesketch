import CSketchSession

/// Pristine real-runtime retirement plus a Swift ordering oracle. Actual poisoned-session
/// drain is separately exercised in services tests; this does not simulate C poisoning.
@MainActor enum NativeRetirementSmoke {
  static func run() throws {
    let runtime = try NativeRuntime(services: false)
    let session = try RustApplicationSession(runtime: runtime)
    runtime.retire(session)
    var replies = 0
    var failed = false
    precondition(runtime.shutdown(onFailure: { failed = true }) { replies += 1 })
    try SmokeStorageJourney.wait(runtime, stage: "terminal-retirement", editor: session) {
      replies == 1 || failed
    }
    guard !failed, replies == 1, runtime.handle == nil, session.handle == nil else {
      throw NativeBridgeError(
        operation: "terminal-retirement", status: SK_V1_INVALID_STATE.rawValue)
    }
    runtime.pump.drain()
    precondition(replies == 1)
    ordering()
    pumpDecisions()
  }
  private static func ordering() {
    let coordinator = NativeShutdownCoordinator()
    var admissions = 0
    var replies = 0
    var failures = 0
    coordinator.request(completion: { replies += 1 }, failure: { failures += 1 })
    coordinator.advance(retired: false) {
      admissions += 1
      return SK_V1_BUSY.rawValue
    }
    precondition(admissions == 0 && replies == 0 && failures == 0)
    coordinator.advance(retired: true) {
      admissions += 1
      return SK_V1_OK.rawValue
    }
    coordinator.advance(retired: true) {
      admissions += 1
      return SK_V1_OK.rawValue
    }
    coordinator.finish()
    coordinator.finish()
    precondition(admissions == 1 && replies == 1 && failures == 0)
  }
  private static func pumpDecisions() {
    var state = SketchRuntimeStateV1()
    state.wake_after_ms = UInt64.max
    precondition(NativePumpDecision(state: state, retiring: false).delay == nil)
    precondition(NativePumpDecision(state: state, retiring: true).delay != nil)
    // The old poll snapshot was idle; retirement/shutdown generated new pending work.
    state.needs_poll = 1
    precondition(NativePumpDecision(state: state, retiring: false).delay != nil)
    state.needs_poll = 0
    state.stopped = 1
    precondition(NativePumpDecision(state: state, retiring: false).delay == nil)
    state.ready_events = 1
    precondition(NativePumpDecision(state: state, retiring: false).delay != nil)
    state.ready_events = 0
    state.retained_events = 1
    precondition(NativePumpDecision(state: state, retiring: false).delay != nil)
  }
}
