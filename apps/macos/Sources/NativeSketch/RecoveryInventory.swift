import AppKit
import CSketchSession

struct RecoveryRow {
  let value: SketchRecoveryItemV1
  var name: String {
    var bytes = value.display_name
    return withUnsafeBytes(of: &bytes) {
      String(decoding: $0.prefix(min(Int(value.display_name_length), $0.count)), as: UTF8.self)
    }
  }
}

@MainActor final class RecoveryInventory {
  private unowned let runtime: NativeRuntime
  private var eventID: UInt64?
  private var request: UInt64 = 0
  private var listOperation: UInt64?
  private(set) var rows: [RecoveryRow] = []
  private(set) var info = SketchRecoveryInfoV1()
  var onChange: (() -> Void)?
  var onFailure: ((SketchFailureV1) -> Void)?
  init(runtime: NativeRuntime) { self.runtime = runtime }

  func refresh() {
    var value = SketchMaintenanceV1()
    value.tag = SK_V1_RECOVERY_LIST.rawValue
    listOperation = submit(value)
  }
  func discard(_ item: UInt64) {
    var value = SketchMaintenanceV1()
    value.tag = SK_V1_RECOVERY_DISCARD.rawValue
    value.item_id = item
    if submit(value) != nil { dismiss() }
  }
  func retry(window: UInt64) {
    var value = SketchMaintenanceV1()
    value.tag = SK_V1_RECOVERY_RETRY.rawValue
    value.window = window
    _ = submit(value)
  }
  func receive(_ event: SketchEventV1) {
    guard event.operation_id == listOperation, onChange != nil else {
      release(event.event_id)
      return
    }
    var count: UInt32 = 0
    var nextInfo = SketchRecoveryInfoV1()
    let queried = sketch_runtime_v1_recovery_copy(
      runtime.handle, event.event_id, nil, 0, &count, &nextInfo)
    guard queried == SK_V1_OK.rawValue || queried == SK_V1_CAPACITY.rawValue, count <= 20 else {
      NativeDiagnostics.failure("recovery-count", status: queried)
      release(event.event_id)
      return
    }
    var items = [SketchRecoveryItemV1](repeating: SketchRecoveryItemV1(), count: Int(count))
    let copied = items.withUnsafeMutableBufferPointer {
      sketch_runtime_v1_recovery_copy(
        runtime.handle, event.event_id, $0.baseAddress, UInt32($0.count), &count, &nextInfo)
    }
    guard copied == SK_V1_OK.rawValue else {
      NativeDiagnostics.failure("recovery-copy", status: copied)
      release(event.event_id)
      return
    }
    dismiss()
    eventID = event.event_id
    rows = items.prefix(Int(count)).map { RecoveryRow(value: $0) }
    info = nextInfo
    onChange?()
  }
  func completed(_ event: SketchEventV1) {
    if event.failure.domain != SK_V1_FAILURE_NONE.rawValue { onFailure?(event.failure) }
    if onChange != nil { refresh() }
  }
  func dismiss() {
    if let eventID { release(eventID) }
    eventID = nil
    rows = []
  }
  private func submit(_ value: SketchMaintenanceV1) -> UInt64? {
    var value = value
    var operation: UInt64 = 0
    guard request < UInt64.max else { return nil }
    request += 1
    let status = sketch_runtime_v1_maintenance(runtime.handle, request, &value, &operation)
    guard status == SK_V1_OK.rawValue else {
      NativeDiagnostics.failure("recovery-maintenance", status: status)
      return nil
    }
    runtime.pump.schedule()
    return operation
  }
  private func release(_ id: UInt64) {
    let status = sketch_runtime_v1_release_event(runtime.handle, id)
    if status != SK_V1_OK.rawValue {
      NativeDiagnostics.failure("inventory-release", status: status)
    }
  }
}
