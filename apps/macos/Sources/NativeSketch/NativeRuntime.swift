import AppKit
import CSketchSession

@MainActor final class NativeRuntime {
  private(set) var handle: OpaquePointer?
  let grants = NativeAccessGrants()
  lazy var pump = RuntimeEventPump(runtime: self)
  lazy var recovery = RecoveryInventory(runtime: self)
  var onFailure: ((SketchFailureV1) -> Void)?
  var onShutdown: (() -> Void)?
  private var sessions: [UInt64: WeakSession] = [:]
  private var retiring: [RustApplicationSession] = []
  private var orphaned: [OpaquePointer] = []

  init(root: URL? = nil, services: Bool = true) throws {
    let base =
      root
      ?? FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask)[0]
      .appendingPathComponent("NativeSketch")
    var config = SketchRuntimeConfigV1()
    config.struct_size = UInt32(MemoryLayout<SketchRuntimeConfigV1>.size)
    let status = withNativeText(services ? base.appendingPathComponent("Recovery").path : "") {
      recovery in
      withNativeText(services ? base.appendingPathComponent("Diagnostics").path : "") {
        diagnostics in
        config.recovery_root = recovery
        config.diagnostics_root = diagnostics
        return sketch_runtime_v1_create(&config, &handle)
      }
    }
    guard status == SK_V1_OK.rawValue, handle != nil else {
      throw NativeBridgeError(operation: "runtime-create", status: status)
    }
  }

  func register(_ session: RustApplicationSession) {
    sessions[session.projection.window] = WeakSession(session)
  }
  func session(_ window: UInt64) -> RustApplicationSession? { sessions[window]?.value }
  func unregister(_ window: UInt64) { sessions.removeValue(forKey: window) }
  func retire(_ session: RustApplicationSession) {
    guard !retiring.contains(where: { $0 === session }) else { return }
    retiring.append(session)
    pump.schedule()
  }
  func retainFailedInitialization(_ handle: OpaquePointer) {
    orphaned.append(handle)
    pump.schedule()
  }
  func finishRetirements() {
    retiring.removeAll { $0.detach() }
    orphaned = orphaned.filter { pointer in
      var handle: OpaquePointer? = pointer
      let status = sketch_session_v1_detach(self.handle, &handle)
      if status != SK_V1_OK.rawValue && status != SK_V1_BUSY.rawValue {
        NativeDiagnostics.failure("initialization-retirement", status: status)
      }
      return status != SK_V1_OK.rawValue
    }
  }
  func shutdown(_ completion: @escaping () -> Void) {
    onShutdown = completion
    let status = sketch_runtime_v1_request_shutdown(handle)
    if status != SK_V1_OK.rawValue {
      NativeDiagnostics.failure("shutdown", status: status)
      return
    }
    pump.schedule()
  }
  func finishShutdownIfReady(_ state: SketchRuntimeStateV1) {
    guard state.stopped != 0, handle != nil else { return }
    let status = sketch_runtime_v1_free(&handle)
    guard status == SK_V1_OK.rawValue else {
      if status != SK_V1_BUSY.rawValue { NativeDiagnostics.failure("runtime-free", status: status) }
      return
    }
    let callback = onShutdown
    onShutdown = nil
    callback?()
  }
}

@MainActor private final class WeakSession {
  weak var value: RustApplicationSession?
  init(_ value: RustApplicationSession) { self.value = value }
}
