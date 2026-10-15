import AppKit
import Metal
import QuartzCore

@MainActor final class MetalSurface {
  let layer = CAMetalLayer()
  let device: any MTLDevice
  let queue: any MTLCommandQueue
  private(set) var generation: UInt64 = 1
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
  func invalidateGeneration() { generation += 1 }
  func resize(_ size: CGSize, scale: CGFloat) {
    let pixels = CGSize(width: ceil(size.width * scale), height: ceil(size.height * scale))
    guard layer.drawableSize != pixels || layer.contentsScale != scale else { return }
    generation += 1
    layer.contentsScale = scale
    layer.drawableSize = pixels
  }
}
