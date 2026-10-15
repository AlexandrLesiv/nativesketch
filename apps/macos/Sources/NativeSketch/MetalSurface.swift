import AppKit
import Metal
import QuartzCore

@MainActor final class MetalSurface {
  let layer = CAMetalLayer()
  let device: any MTLDevice
  let queue: any MTLCommandQueue
  private var sizing = RenderSurfaceState()
  var generation: UInt64 { sizing.generation }
  init() throws {
    guard let device = MTLCreateSystemDefaultDevice(), let queue = device.makeCommandQueue() else {
      throw NativeBridgeError(operation: "metal-device", status: 9)
    }
    self.device = device
    self.queue = queue
    layer.device = device
    layer.pixelFormat = .bgra8Unorm
    layer.framebufferOnly = false
    layer.maximumDrawableCount = 2
    layer.allowsNextDrawableTimeout = true
    layer.colorspace = CGColorSpace(name: CGColorSpace.sRGB)
    layer.isOpaque = true
  }
  func invalidateGeneration() { sizing.invalidate() }
  func resize(_ size: CGSize, scale: CGFloat) {
    guard sizing.update(size, scale: scale) else { return }
    layer.contentsScale = scale
    layer.drawableSize = sizing.pixels
  }
}
