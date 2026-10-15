import AppKit
import CSketchSession

#if DEBUG && NATIVE_TEST_CONTROLS
  import CSketchSessionTest
#endif

@MainActor final class RustApplicationSession {
  let runtime: NativeRuntime
  private(set) var handle: OpaquePointer?
  private(set) var projection = SketchProjectionV1()
  private(set) var unavailable = false
  private var request: UInt64 = 0
  #if DEBUG && NATIVE_TEST_CONTROLS
    private var injectedCloseFailure = false
    func injectCloseFailureIfRequested() {
      guard !injectedCloseFailure, CommandLine.arguments.contains("--test-poison-on-close")
      else { return }
      injectedCloseFailure = true
      let status = sketch_runtime_v1_test_poison(runtime.handle)
      _ = accept(status, SketchOutcomeV1())
    }
  #endif
  var onOutcome: ((SketchOutcomeV1) -> Void)?
  var onFailure: ((UInt32) -> Void)?

  init(runtime: NativeRuntime) throws {
    self.runtime = runtime
    var config = SketchSessionConfigV1()
    config.capabilities = SK_V1_CAP_EDIT.rawValue | SK_V1_CAP_EXPORT.rawValue
    config.tool = SK_V1_PEN.rawValue
    config.brush = SketchBrushV1(red: 0, green: 0, blue: 0, alpha: 1, width: 6)
    let status = sketch_session_v1_create(runtime.handle, &config, &handle)
    guard status == SK_V1_OK.rawValue else {
      throw NativeBridgeError(operation: "session-create", status: status)
    }
    let projected = sketch_session_v1_projection(runtime.handle, handle, &projection)
    guard projected == SK_V1_OK.rawValue else {
      let detached = sketch_session_v1_detach(runtime.handle, &handle)
      if detached != SK_V1_OK.rawValue, let handle {
        NativeDiagnostics.failure("failed-session-retirement", status: detached)
        runtime.retainFailedInitialization(handle)
        self.handle = nil
      }
      throw NativeBridgeError(operation: "session-projection", status: projected)
    }
    runtime.register(self)
  }

  func nextContext() -> SketchContextV1 {
    precondition(request < UInt64.max, "Native request identity exhausted")
    request += 1
    return SketchContextV1(
      session: projection.session, request: request, interaction: projection.interaction)
  }

  @discardableResult func send(_ command: SessionCommand) -> Bool {
    guard !unavailable, handle != nil else { return false }
    var value = command.value
    var outcome = SketchOutcomeV1()
    let status = sketch_session_v1_dispatch(runtime.handle, handle, nextContext(), &value, &outcome)
    let accepted = accept(status, outcome)
    runtime.pump.schedule()
    return accepted
  }

  @discardableResult func append(_ samples: [SketchSampleV1], pointer: UInt64) -> Bool {
    guard !unavailable, samples.count <= 4096 else { return false }
    var outcome = SketchOutcomeV1()
    let context = nextContext()
    let status = samples.withUnsafeBufferPointer {
      sketch_session_v1_input(
        runtime.handle, handle, context, pointer, $0.baseAddress, UInt32($0.count), &outcome)
    }
    return accept(status, outcome)
  }

  @discardableResult func reply(_ token: SketchEffectTokenV1, _ reply: SketchReplyTagV1) -> Bool {
    var outcome = SketchOutcomeV1()
    let status = sketch_session_v1_reply(
      runtime.handle, handle, nextContext(), token, reply.rawValue, &outcome)
    let accepted = accept(status, outcome)
    runtime.pump.schedule()
    return accepted
  }

  func supply(_ url: URL, token: SketchEffectTokenV1) -> Bool {
    do {
      let grant = try runtime.grants.retain(url)
      var operation: UInt64 = 0
      let status = withNativeText(url.path) { text in
        var location = SketchLocationV1(grant_id: grant, path: text, provider: 0, reserved: 0)
        return sketch_session_v1_supply_location(
          runtime.handle, handle, nextContext(), token, &location, &operation)
      }
      if status != SK_V1_OK.rawValue {
        runtime.grants.release(grant)
        return accept(status, SketchOutcomeV1())
      }
      refreshProjection()
      runtime.pump.schedule()
      return true
    } catch {
      NativeDiagnostics.failure(
        "location-grant", status: SK_V1_CAPACITY.rawValue, session: projection.session)
      onFailure?(SK_V1_CAPACITY.rawValue)
      return false
    }
  }

  func apply(_ event: UInt64) -> UInt32 {
    var outcome = SketchOutcomeV1()
    let status = sketch_session_v1_apply_event(runtime.handle, handle, event, &outcome)
    if status == SK_V1_OK.rawValue { _ = accept(status, outcome) }
    return status
  }

  func available(_ action: UInt32) -> Bool {
    guard !unavailable else { return false }
    var result = SketchAvailabilityV1()
    let status = sketch_session_v1_availability(runtime.handle, handle, action, &result)
    return status == SK_V1_OK.rawValue && result.enabled != 0
  }

  func refreshProjection() {
    var value = SketchProjectionV1()
    let status = sketch_session_v1_projection(runtime.handle, handle, &value)
    if status == SK_V1_OK.rawValue {
      projection = value
    } else {
      _ = accept(status, SketchOutcomeV1())
    }
  }

  func registerRecovery(name: String) {
    var identity = UUID().uuid
    let status = withUnsafeBytes(of: &identity) { bytes in
      withNativeText(name) { text in
        sketch_session_v1_recovery_identity(
          runtime.handle, handle, projection.session,
          bytes.bindMemory(to: UInt8.self).baseAddress, text)
      }
    }
    if status != SK_V1_OK.rawValue {
      NativeDiagnostics.failure("recovery-register", status: status, session: projection.session)
    }
    runtime.pump.schedule()
  }

  @discardableResult func detach() -> Bool {
    guard handle != nil else { return true }
    let window = projection.window
    let status = sketch_session_v1_detach(runtime.handle, &handle)
    if status == SK_V1_OK.rawValue {
      runtime.unregister(window)
      return true
    }
    if status != SK_V1_BUSY.rawValue {
      NativeDiagnostics.failure("session-detach", status: status, session: projection.session)
    }
    return false
  }

  private func accept(_ status: UInt32, _ outcome: SketchOutcomeV1) -> Bool {
    guard status == SK_V1_OK.rawValue else {
      if status == SK_V1_PANIC.rawValue || status == SK_V1_POISONED.rawValue { unavailable = true }
      NativeDiagnostics.failure("session-command", status: status, session: projection.session)
      onFailure?(status)
      return false
    }
    projection = outcome.projection
    onOutcome?(outcome)
    return true
  }
}
