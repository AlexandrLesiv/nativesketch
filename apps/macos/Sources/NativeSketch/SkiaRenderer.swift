import AppKit
import CSketchSession
import Metal

@MainActor final class SkiaRenderer {
  private var handle: OpaquePointer?
  init(surface: MetalSurface) throws {
    let status = sketch_renderer_v1_create(
      Unmanaged.passUnretained(surface.device as AnyObject).toOpaque(),
      Unmanaged.passUnretained(surface.queue as AnyObject).toOpaque(), &handle)
    guard status == SK_V1_OK.rawValue else {
      throw NativeBridgeError(operation: "renderer-create", status: status)
    }
  }
  func close() {
    let status = sketch_renderer_v1_free(&handle)
    if status != SK_V1_OK.rawValue { NativeDiagnostics.failure("renderer-free", status: status) }
  }
  func draw(canvas: CanvasView, surface: MetalSurface, texture: any MTLTexture) throws
    -> SketchRenderStatsV1
  {
    let state = canvas.editor.projection
    var request = SketchRenderRequestV1()
    request.struct_size = UInt32(MemoryLayout<SketchRenderRequestV1>.size)
    request.focused = canvas.window?.firstResponder === canvas ? 1 : 0
    request.session = state.session
    request.revision = state.revision
    request.scene_generation = state.scene_generation
    request.surface_generation = surface.generation
    request.logical_width = canvas.bounds.width
    request.logical_height = canvas.bounds.height
    request.pixel_width = UInt32(texture.width)
    request.pixel_height = UInt32(texture.height)
    request.zoom = canvas.zoom
    request.offset_x = canvas.offset.x
    request.offset_y = canvas.offset.y
    canvas.effectiveAppearance.performAsCurrentDrawingAppearance {
      request.selection_rgba = rgba(.systemBlue)
      request.focus_rgba = rgba(.keyboardFocusIndicatorColor)
    }
    var stats = SketchRenderStatsV1()
    let status = sketch_renderer_v1_draw(
      canvas.editor.runtime.handle, canvas.editor.handle, handle,
      Unmanaged.passUnretained(texture as AnyObject).toOpaque(), &request, &stats)
    guard status == SK_V1_OK.rawValue else {
      throw NativeBridgeError(operation: "renderer-draw", status: status)
    }
    return stats
  }
  private func rgba(_ color: NSColor) -> (Double, Double, Double, Double) {
    let value = color.usingColorSpace(.sRGB) ?? .black
    return (value.redComponent, value.greenComponent, value.blueComponent, value.alphaComponent)
  }
}
