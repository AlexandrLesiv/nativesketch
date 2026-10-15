import AppKit
import CSketchFFI

// All adapter/editor access is confined to AppKit's main actor. Core owns editing,
// history, hit testing, pressure geometry, and persistence. This painter is provisional.
@MainActor final class LegacyEditorBridge {
  private(set) var handle: OpaquePointer?
  var onFailure: ((UInt32) -> Void)?
  var unavailable: Bool { handle == nil }
  init() { reset() }
  isolated deinit { if let handle { sketch_editor_free(handle) } }
  func reset() {
    if let handle { sketch_editor_free(handle) }
    handle = nil
    precondition(sketch_editor_create(&handle) == 0)
  }
  // Test seam also exercises panic status without adding a production Rust hook.
  @discardableResult func observe(_ status: UInt32) -> UInt32 {
    if status == 6, let old = handle {
      handle = nil
      sketch_editor_free(old)
      onFailure?(status)
    } else if status != 0 && status != 6 {
      onFailure?(status)
    }
    return status
  }
  var state: SketchState {
    guard handle != nil else { return SketchState() }
    var state = SketchState()
    guard observe(sketch_editor_state(handle, &state)) == 0 else { return SketchState() }
    return state
  }
  func file(_ url: URL, save: Bool) -> UInt32 {
    guard handle != nil else { return 6 }
    let bytes = Array(url.path.utf8)
    return observe(
      bytes.withUnsafeBufferPointer {
        save
          ? sketch_document_save(handle, $0.baseAddress, $0.count)
          : sketch_document_load(handle, $0.baseAddress, $0.count)
      })
  }
  func commands() -> [SketchPathCommand] {
    guard handle != nil else { return [] }
    var count = 0
    guard observe(sketch_render_copy(handle, nil, 0, &count)) == 0 else { return [] }
    var commands = Array(repeating: SketchPathCommand(), count: count)
    let result = commands.withUnsafeMutableBufferPointer {
      sketch_render_copy(handle, $0.baseAddress, $0.count, &count)
    }
    return observe(result) == 0 ? commands : []
  }
}
