import CSketchSession

@MainActor final class SessionScene {
  private unowned let session: RustApplicationSession
  init(session: RustApplicationSession) { self.session = session }

  func bounds() -> SketchBoundsV1? {
    var bounds = SketchBoundsV1()
    let status = sketch_session_v1_selected_bounds(session.runtime.handle, session.handle, &bounds)
    if status == SK_V1_NOT_FOUND.rawValue { return nil }
    guard status == SK_V1_OK.rawValue else {
      NativeDiagnostics.failure(
        "selected-bounds", status: status, session: session.projection.session)
      return nil
    }
    return bounds
  }

  func commands() -> [SketchPathV1] {
    var handle: OpaquePointer?
    let status = sketch_session_v1_scene(session.runtime.handle, session.handle, &handle)
    guard status == SK_V1_OK.rawValue, handle != nil else {
      NativeDiagnostics.failure("scene-create", status: status, session: session.projection.session)
      return []
    }
    defer {
      let released = sketch_scene_v1_free(&handle)
      if released != SK_V1_OK.rawValue { NativeDiagnostics.failure("scene-free", status: released) }
    }
    var info = SketchSceneInfoV1()
    guard sketch_scene_v1_info(handle, &info) == SK_V1_OK.rawValue,
      info.command_count <= UInt64(UInt32.max), info.command_count <= UInt64(Int.max)
    else { return [] }
    var commands = [SketchPathV1](repeating: SketchPathV1(), count: Int(info.command_count))
    var count: UInt32 = 0
    let copied = commands.withUnsafeMutableBufferPointer {
      sketch_scene_v1_copy(handle, $0.baseAddress, UInt32($0.count), &count)
    }
    guard copied == SK_V1_OK.rawValue else {
      NativeDiagnostics.failure("scene-copy", status: copied, session: session.projection.session)
      return []
    }
    return Array(commands.prefix(Int(count)))
  }
}
