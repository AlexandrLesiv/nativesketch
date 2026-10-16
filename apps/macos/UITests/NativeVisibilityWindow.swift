import AppKit
import ApplicationServices
import XCTest

/// Observe and restore the real disposable window; never modifies renderer or editor state.
@MainActor final class NativeVisibilityWindow {
  private let window: AXUIElement

  init() throws {
    XCTAssertTrue(AXIsProcessTrusted(), "Stable runner must retain Accessibility permission")
    let path = try XCTUnwrap(ProcessInfo.processInfo.environment["NATIVE_SKETCH_APP"])
    let expected = URL(fileURLWithPath: path).resolvingSymlinksInPath()
    let identifier = try XCTUnwrap(Bundle(url: expected)?.bundleIdentifier)
    XCTAssertTrue(identifier.hasPrefix("dev.nativesketch.e2e."))
    XCTAssertEqual(expected.lastPathComponent, "Subject.app")
    let candidates = NSRunningApplication.runningApplications(withBundleIdentifier: identifier)
      .filter { $0.bundleURL?.resolvingSymlinksInPath() == expected }
    XCTAssertEqual(candidates.count, 1)
    let subject = try XCTUnwrap(candidates.first)
    let app = AXUIElementCreateApplication(subject.processIdentifier)
    var raw: CFTypeRef?
    XCTAssertEqual(
      AXUIElementCopyAttributeValue(app, kAXWindowsAttribute as CFString, &raw), .success)
    let windows = try XCTUnwrap(raw as? [AXUIElement])
    XCTAssertEqual(windows.count, 1, "Require the single disposable editor window")
    window = try XCTUnwrap(windows.first)
  }

  var minimized: Bool? {
    var raw: CFTypeRef?
    guard AXUIElementCopyAttributeValue(window, kAXMinimizedAttribute as CFString, &raw) == .success
    else { return nil }
    return raw as? Bool
  }

  func restore() throws {
    let result = AXUIElementSetAttributeValue(
      window, kAXMinimizedAttribute as CFString, kCFBooleanFalse)
    guard result == .success else {
      throw NSError(domain: "NativeVisibilityRestore", code: Int(result.rawValue))
    }
  }
}
