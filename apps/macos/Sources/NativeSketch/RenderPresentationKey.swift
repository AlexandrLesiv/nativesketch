import AppKit

/// Compare current pixel inputs, including changes made before updateLayer is delivered.
struct RenderPresentationKey: Equatable {
  let session: UInt64
  let scene: UInt64
  let surface: UInt64
  let request: UInt64
  let size: CGSize
  let scale: CGFloat
  let zoom: Double
  let offset: CGPoint
  let focused: Bool
  let colors: [CGFloat]

  @MainActor init(canvas: CanvasView, surface: UInt64, request: UInt64) {
    session = canvas.editor.projection.session
    scene = canvas.editor.projection.scene_generation
    self.surface = surface
    self.request = request
    size = canvas.bounds.size
    scale = canvas.window?.backingScaleFactor ?? 1
    zoom = canvas.zoom
    offset = canvas.offset
    focused = canvas.window?.firstResponder === canvas
    var resolved: [CGFloat] = []
    canvas.effectiveAppearance.performAsCurrentDrawingAppearance {
      for color in [NSColor.systemBlue, NSColor.keyboardFocusIndicatorColor] {
        let value = color.usingColorSpace(.sRGB) ?? .black
        resolved += [
          value.redComponent, value.greenComponent, value.blueComponent, value.alphaComponent,
        ]
      }
    }
    colors = resolved
  }
}
