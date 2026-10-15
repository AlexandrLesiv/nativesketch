import AppKit
import CSketchSession

/// One bounded main-loop source. Never drain or capture snapshots inside an input callback.
@MainActor final class RuntimeEventPump {
  private unowned let runtime: NativeRuntime
  private var timer: Timer?
  private var retained: [UInt64: SketchEventV1] = [:]
  init(runtime: NativeRuntime) { self.runtime = runtime }

  func schedule(after interval: TimeInterval = 0) {
    guard runtime.handle != nil else { return }
    timer?.invalidate()
    timer = Timer.scheduledTimer(withTimeInterval: max(0.001, interval), repeats: false) {
      [weak self] _ in
      MainActor.assumeIsolated { self?.drain() }
    }
  }

  func drain() {
    timer = nil
    guard runtime.handle != nil else { return }
    var state = SketchRuntimeStateV1()
    var count: UInt32 = 0
    var events = [SketchEventV1](repeating: SketchEventV1(), count: 8)
    let status = events.withUnsafeMutableBufferPointer {
      sketch_runtime_v1_poll(runtime.handle, $0.baseAddress, UInt32($0.count), &count, &state)
    }
    guard status == SK_V1_OK.rawValue else {
      NativeDiagnostics.failure("poll", status: status)
      return
    }
    for event in events.prefix(Int(count)) { retained[event.event_id] = event }
    let deadline = ProcessInfo.processInfo.systemUptime + 0.002
    for (id, event) in retained.sorted(by: { $0.key < $1.key }).prefix(8) {
      if ProcessInfo.processInfo.systemUptime >= deadline { break }
      if process(event) { retained.removeValue(forKey: id) }
    }
    runtime.finishRetirements()
    let idleStatus = sketch_runtime_v1_idle(runtime.handle, &state)
    if idleStatus != SK_V1_OK.rawValue && idleStatus != SK_V1_BUSY.rawValue {
      NativeDiagnostics.failure("idle", status: idleStatus)
    }
    // Retirement can queue work/admit shutdown after the poll snapshot. A poisoned idle
    // call leaves its output untouched; state() remains the supported lifecycle query.
    let stateStatus = sketch_runtime_v1_state(runtime.handle, &state)
    guard stateStatus == SK_V1_OK.rawValue else {
      NativeDiagnostics.failure("runtime-state", status: stateStatus)
      return
    }
    runtime.finishShutdownIfReady(state)
    if state.needs_poll != 0 || state.ready_events != 0 || runtime.awaitingRetirement {
      schedule(after: 0.016)
    } else if state.wake_after_ms != UInt64.max {
      // Runtime can request an idle turn; avoid spinning while active ink owns a retained event.
      schedule(after: max(0.016, Double(state.wake_after_ms) / 1000))
    }
  }

  private func process(_ event: SketchEventV1) -> Bool {
    switch event.kind {
    case SK_V1_OWNER_READY.rawValue:
      guard let session = runtime.session(event.window) else { return release(event) }
      let status = session.apply(event.event_id)
      if status == SK_V1_BUSY.rawValue { return false }
      if status == SK_V1_STALE.rawValue { return release(event) }
      if status != SK_V1_OK.rawValue {
        NativeDiagnostics.failure("apply-event", status: status, session: event.session)
        return false
      }
      return true
    case SK_V1_GRANT_RELEASE.rawValue:
      runtime.grants.release(event.grant_id)
      return release(event)
    case SK_V1_RECOVERY_READY.rawValue:
      runtime.recovery.receive(event)
      return true  // Inventory now owns the issued event until its presenter releases it.
    case SK_V1_MAINTENANCE_COMPLETE.rawValue:
      runtime.recovery.completed(event)
      return release(event)
    case SK_V1_RUNTIME_FAILURE.rawValue:
      runtime.onFailure?(event.failure)
      return release(event)
    case SK_V1_SHUTDOWN_COMPLETE.rawValue: return release(event)
    default:
      NativeDiagnostics.failure("unknown-event", status: event.kind)
      return release(event)
    }
  }

  private func release(_ event: SketchEventV1) -> Bool {
    let status = sketch_runtime_v1_release_event(runtime.handle, event.event_id)
    if status != SK_V1_OK.rawValue {
      NativeDiagnostics.failure("release-event", status: status, session: event.session)
    }
    return status == SK_V1_OK.rawValue
  }
}
