import CSketchFFI
import UIKit
import UniformTypeIdentifiers

extension DrawingView {
  #if !os(visionOS)
    func pressure(_ t: UITouch) -> Double? {
      t.type == .pencil && t.maximumPossibleForce > 0
        ? Double(t.force / t.maximumPossibleForce) : nil
    }
    func navigationPair() -> (CGPoint, Double)? {
      let points = navigationTouches.filter { contacts.contains($0) }.map { $0.location(in: self) }
      guard points.count >= 2 else { return nil }
      let a = points[0]
      let b = points[1]
      return (CGPoint(x: (a.x + b.x) / 2, y: (a.y + b.y) / 2), hypot(a.x - b.x, a.y - b.y))
    }
    func beginNavigation() {
      cancel()
      navigating = true
      owner.refresh()
    }
    func inputBegan(_ touches: Set<UITouch>, with event: UIEvent?) {
      contacts.formUnion(touches)
      if contacts.count > 1 {
        if !navigating {
          navigationTouches = Array(contacts.prefix(2))
          beginNavigation()
          previousNavigation = navigationPair()
        }
        return
      }
      guard !navigating, let t = touches.first else { return }
      ownerTouch = t
      begin(
        at: t.location(in: self), timestamp: t.timestamp, pressure: pressure(t),
        pencil: t.type == .pencil)
    }
    func inputMoved(_ touches: Set<UITouch>, with event: UIEvent?) {
      if navigating {
        if let current = navigationPair(), let previous = previousNavigation {
          let d = document(previous.0)
          if previous.1 > 1 { zoom = navigationZoom(zoom * current.1 / previous.1) }
          offset = CGPoint(x: current.0.x - d.x * zoom, y: current.0.y - d.y * zoom)
          owner.refresh()
        }
        previousNavigation = navigationPair()
        return
      }
      guard let t = ownerTouch, touches.contains(t) else { return }
      for sample in event?.coalescedTouches(for: t) ?? [t] {
        move(to: sample.location(in: self), timestamp: sample.timestamp, pressure: pressure(sample))
      }
    }
    func inputEnded(_ touches: Set<UITouch>, with event: UIEvent?) {
      if !navigating, let t = ownerTouch, touches.contains(t) {
        move(to: t.location(in: self), timestamp: t.timestamp, pressure: pressure(t))
        end()
      }
      contacts.subtract(touches)
      previousNavigation = navigationPair()
      if contacts.isEmpty {
        navigating = false
        previousNavigation = nil
        navigationTouches.removeAll()
      }
    }
  #endif
}
