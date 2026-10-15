import ApplicationServices
import XCTest

/// Only native synthetic mouse input. It never calls an editor command or replay hook.
@MainActor final class NativeHeldPointer {
  private let pid: pid_t
  private var point = CGPoint.zero
  private var held = false

  init(pid: pid_t) throws {
    guard CGPreflightPostEventAccess() else {
      throw NSError(
        domain: "NativeGestureEnvironment", code: 1,
        userInfo: [
          NSLocalizedDescriptionKey:
            "The XCTest runner lacks native event-posting permission. No permission was requested; this journey remains unverified."
        ])
    }
    self.pid = pid
  }

  func begin(in bounds: CGRect) throws {
    XCTAssertFalse(held)
    XCTAssertGreaterThan(bounds.width, 300)
    XCTAssertGreaterThan(bounds.height, 200)
    let start = CGPoint(x: bounds.minX + bounds.width * 0.4, y: bounds.minY + bounds.height * 0.6)
    try post(.leftMouseDown, at: start)
    try post(.leftMouseDragged, at: CGPoint(x: start.x + 55, y: start.y + 30))
  }

  func release() throws {
    if held { try post(.leftMouseUp, at: point) }
  }

  private func post(_ type: CGEventType, at location: CGPoint) throws {
    guard
      let event = CGEvent(
        mouseEventSource: nil, mouseType: type, mouseCursorPosition: location, mouseButton: .left)
    else {
      throw NSError(domain: "NativeGestureEvent", code: 1)
    }
    event.setIntegerValueField(.mouseEventClickState, value: 1)
    event.postToPid(pid)
    point = location
    held = type != .leftMouseUp
  }
}
