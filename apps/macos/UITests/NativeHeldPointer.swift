import ApplicationServices
import XCTest

/// Only native synthetic mouse input. It never calls an editor command or replay hook.
@MainActor final class NativeHeldPointer {
  private let pid: pid_t
  private var point = CGPoint.zero
  private var held = false
  private var windowNumber: Int64 = 0

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

  func begin(in bounds: CGRect, windowBounds: CGRect) throws {
    XCTAssertFalse(held)
    XCTAssertGreaterThan(bounds.width, 300)
    XCTAssertGreaterThan(bounds.height, 200)
    windowNumber = try ownedWindowNumber(bounds: windowBounds)
    XCTContext.runActivity(named: "PID-bound native mouse routing") { activity in
      let evidence = XCTAttachment(
        string:
          "pid=\(pid), windowNumber=\(windowNumber), windowBounds=\(windowBounds), canvasBounds=\(bounds)"
      )
      evidence.lifetime = .keepAlways
      activity.add(evidence)
    }
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
    // PostToPid bypasses Window Server hit-testing/annotation. Supply the documented
    // window fields instead of relying on an unannotated factory event being routed.
    event.setIntegerValueField(.mouseEventWindowUnderMousePointer, value: windowNumber)
    event.setIntegerValueField(
      .mouseEventWindowUnderMousePointerThatCanHandleThisEvent, value: windowNumber)
    event.setIntegerValueField(.eventTargetUnixProcessID, value: Int64(pid))
    event.postToPid(pid)
    point = location
    held = type != .leftMouseUp
  }

  private func ownedWindowNumber(bounds: CGRect) throws -> Int64 {
    let windows =
      CGWindowListCopyWindowInfo(.optionOnScreenOnly, kCGNullWindowID)
      as? [[String: Any]] ?? []
    let matches = windows.filter { entry in
      guard (entry[kCGWindowOwnerPID as String] as? NSNumber)?.int32Value == pid,
        (entry[kCGWindowLayer as String] as? NSNumber)?.intValue == 0,
        let encoded = entry[kCGWindowBounds as String] as? NSDictionary,
        let actual = CGRect(dictionaryRepresentation: encoded)
      else { return false }
      return actual == bounds
    }
    guard matches.count == 1,
      let number = matches[0][kCGWindowNumber as String] as? NSNumber
    else {
      throw NSError(
        domain: "NativeGestureWindowIdentity", code: 1,
        userInfo: [
          NSLocalizedDescriptionKey:
            "Require one on-screen window matching the owned PID and exact AX bounds"
        ])
    }
    return number.int64Value
  }
}
