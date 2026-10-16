import CSketchFFI
import UIKit
import UniformTypeIdentifiers

// The entire adapter is main-actor isolated. Rust owns document/edit/history semantics.
@MainActor final class MobileEditor {
  var handle: OpaquePointer?
  var failure: ((UInt32) -> Void)?
  init() {
    precondition(sketch_abi_version() == 2)
    reset()
  }
  isolated deinit { if let handle { sketch_editor_free(handle) } }
  func reset() {
    if let handle { sketch_editor_free(handle) }
    handle = nil
    precondition(sketch_editor_create(&handle) == 0)
  }
  @discardableResult func check(_ code: UInt32) -> Bool {
    if code == 6, let old = handle {
      handle = nil
      sketch_editor_free(old)
    }
    if code != 0 { failure?(code) }
    return code == 0
  }
  var state: SketchState {
    var s = SketchState()
    if handle != nil { _ = check(sketch_editor_state(handle, &s)) }
    return s
  }
  func commands() -> [SketchPathCommand] {
    guard handle != nil else { return [] }
    var count = 0
    guard check(sketch_render_copy(handle, nil, 0, &count)) else { return [] }
    var result = Array(repeating: SketchPathCommand(), count: count)
    return result.withUnsafeMutableBufferPointer {
      check(sketch_render_copy(handle, $0.baseAddress, $0.count, &count))
    } ? result : []
  }
  func snapshot() -> (Data, UInt64)? {
    var count = 0
    var revision: UInt64 = 0
    guard check(sketch_document_export(handle, nil, 0, &count, &revision)) else { return nil }
    var data = Data(count: count)
    guard
      data.withUnsafeMutableBytes({
        check(
          sketch_document_export(
            handle, $0.bindMemory(to: UInt8.self).baseAddress, count, &count, &revision))
      })
    else { return nil }
    return (data, revision)
  }
  func load(_ data: Data) -> Bool {
    data.withUnsafeBytes {
      check(sketch_document_import(handle, $0.bindMemory(to: UInt8.self).baseAddress, $0.count))
    }
  }
}
