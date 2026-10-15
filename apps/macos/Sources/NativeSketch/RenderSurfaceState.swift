import Foundation

/// Logical geometry and density both participate in the native surface identity.
struct RenderSurfaceState {
  private(set) var logical = CGSize.zero
  private(set) var pixels = CGSize.zero
  private(set) var scale: CGFloat = 0
  private(set) var generation: UInt64 = 1

  mutating func invalidate() { generation += 1 }

  mutating func update(_ size: CGSize, scale: CGFloat) -> Bool {
    let pixels = CGSize(width: ceil(size.width * scale), height: ceil(size.height * scale))
    guard logical != size || self.scale != scale || self.pixels != pixels else { return false }
    logical = size
    self.pixels = pixels
    self.scale = scale
    invalidate()
    return true
  }
}
