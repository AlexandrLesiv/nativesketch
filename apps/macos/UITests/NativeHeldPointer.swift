import ApplicationServices
import XCTest

/// Native synthetic mouse input through Window Server, never editor commands.
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
            "The XCTest runner lacks event-posting permission; no permission requested."
        ])
    }
    self.pid = pid
  }

  func begin(in bounds: CGRect, validateHit: (CGPoint) throws -> Void) throws {
    XCTAssertFalse(held)
    XCTAssertGreaterThan(bounds.width, 300)
    XCTAssertGreaterThan(bounds.height, 200)
    let start = CGPoint(x: bounds.minX + bounds.width * 0.4, y: bounds.minY + bounds.height * 0.6)
    let end = CGPoint(x: start.x + 55, y: start.y + 30)
    guard bounds.contains(start), bounds.contains(end) else {
      throw NSError(domain: "NativeGestureBounds", code: 1)
    }
    try validateHit(start)
    try validateHit(end)
    XCTContext.runActivity(named: "Guarded session event-tap native mouse routing") { activity in
      let evidence = XCTAttachment(
        string:
          "frontmost owned pid=\(pid), both points AX-hit verified against owned canvas, bounds=\(bounds)"
      )
      evidence.lifetime = .keepAlways
      activity.add(evidence)
    }
    try post(.leftMouseDown, at: start)
    try post(.leftMouseDragged, at: end)
  }

  func release() throws {
    if held { try post(.leftMouseUp, at: point) }
  }

  private func post(_ type: CGEventType, at location: CGPoint) throws {
    guard
      let event = CGEvent(
        mouseEventSource: nil, mouseType: type, mouseCursorPosition: location, mouseButton: .left)
    else { throw NSError(domain: "NativeGestureEvent", code: 1) }
    event.flags = []
    event.setIntegerValueField(.mouseEventClickState, value: 1)
    // Window Server annotates/dispatches this route. Always balance held input even
    // after intentional deactivation; cleanup must not require the app to be active.
    event.post(tap: .cgSessionEventTap)
    point = location
    held = type != .leftMouseUp
  }
}
