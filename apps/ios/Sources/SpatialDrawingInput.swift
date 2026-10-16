import CSketchFFI
import UIKit
import UniformTypeIdentifiers

extension DrawingView {
  #if os(visionOS)
    // Indirect spatial input may expose zero UITouches. Recognizer locations and
    // translations, not touch counts, own this entire sequence.
    @objc func spatialDrag(_ gesture: UIPanGestureRecognizer) {
      let p = gesture.location(in: self)
      switch gesture.state {
      case .began:
        cancelAll()
        panOrigin = offset
        if owner.tool.selectedSegmentIndex != 2 {
          begin(
            at: p, timestamp: ProcessInfo.processInfo.systemUptime, pressure: nil, pencil: false)
        }
      case .changed:
        if owner.tool.selectedSegmentIndex == 2 {
          let translation = gesture.translation(in: self)
          offset = CGPoint(x: panOrigin.x + translation.x, y: panOrigin.y + translation.y)
          owner.refresh()
        } else {
          move(to: p, timestamp: ProcessInfo.processInfo.systemUptime, pressure: nil)
        }
      case .ended:
        if owner.tool.selectedSegmentIndex == 2 {
          let translation = gesture.translation(in: self)
          offset = CGPoint(x: panOrigin.x + translation.x, y: panOrigin.y + translation.y)
          owner.refresh()
        } else {
          move(to: p, timestamp: ProcessInfo.processInfo.systemUptime, pressure: nil)
          end()
        }
      case .cancelled, .failed: cancelAll()
      default: break
      }
    }
    @objc func spatialTap(_ gesture: UITapGestureRecognizer) {
      guard gesture.state == .ended, owner.tool.selectedSegmentIndex != 2 else { return }
      begin(
        at: gesture.location(in: self), timestamp: ProcessInfo.processInfo.systemUptime,
        pressure: nil, pencil: false)
      end()
    }
  #endif
}
