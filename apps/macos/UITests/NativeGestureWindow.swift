import AppKit
import ApplicationServices
import XCTest

/// Native window/input ownership, with read-only AX observations of the real Rust projection.
@MainActor final class NativeGestureWindow {
  let subject: NSRunningApplication
  let pointer: NativeHeldPointer
  private let window: AXUIElement
  private let canvas: AXUIElement
  private let originalSize: CGSize
  private let focusHelper: NativeFocusHelper

  init(fixture: NativeSessionFixture, file: URL) throws {
    guard AXIsProcessTrusted() else {
      throw NSError(
        domain: "NativeGestureEnvironment", code: 2,
        userInfo: [
          NSLocalizedDescriptionKey:
            "The XCTest runner lacks Accessibility permission. No permission was requested; this journey remains unverified."
        ])
    }
    let path = try XCTUnwrap(ProcessInfo.processInfo.environment["NATIVE_SKETCH_APP"])
    let expected = URL(fileURLWithPath: path).resolvingSymlinksInPath()
    let bundle = try XCTUnwrap(Bundle(url: expected))
    let identifier = try XCTUnwrap(bundle.bundleIdentifier)
    XCTAssertTrue(identifier.hasPrefix("dev.nativesketch.e2e."))
    XCTAssertEqual(expected.lastPathComponent, "Subject.app")
    let candidates = NSRunningApplication.runningApplications(withBundleIdentifier: identifier)
      .filter { $0.bundleURL?.resolvingSymlinksInPath() == expected }
    XCTAssertEqual(candidates.count, 1, "Require exactly the launched disposable subject")
    subject = try XCTUnwrap(candidates.first)
    XCTAssertTrue(subject.isActive)
    XCTAssertTrue(
      file.resolvingSymlinksInPath().path.hasPrefix(
        fixture.root.resolvingSymlinksInPath().path + "/"))
    XCTAssertEqual(
      fixture.runtimeRoot.deletingLastPathComponent().resolvingSymlinksInPath().path,
      fixture.root.resolvingSymlinksInPath().path)
    pointer = try NativeHeldPointer(pid: subject.processIdentifier)
    let app = AXUIElementCreateApplication(subject.processIdentifier)
    let windows = Self.attribute(app, kAXWindowsAttribute) as? [AXUIElement] ?? []
    window = try XCTUnwrap(
      windows.first { element in
        Self.attribute(element, kAXTitleAttribute) as? String == file.lastPathComponent
          + " — Native Sketch"
      }, "Require exact native Save-bound disposable filename")
    canvas = try XCTUnwrap(Self.findCanvas(window))
    originalSize = try Self.size(window)
    focusHelper = try NativeFocusHelper(fixture: fixture, subject: subject)
    XCTAssertEqual(interaction, 0, "Missing observation cannot stand in for idle")
  }

  var interaction: Int? {
    let parts = projection.components(separatedBy: " | ")
    guard let part = parts.first(where: { $0.hasPrefix("Interaction=") }) else { return nil }
    return Int(part.dropFirst("Interaction=".count))
  }

  var projection: String { Self.attribute(canvas, kAXValueAttribute) as? String ?? "" }

  func begin() throws {
    XCTAssertTrue(subject.isActive)
    let origin = try Self.point(canvas)
    try pointer.begin(
      in: CGRect(origin: origin, size: Self.size(canvas)),
      validateHit: validateHit)
  }

  private func validateHit(_ point: CGPoint) throws {
    guard subject.isActive,
      NSWorkspace.shared.frontmostApplication?.processIdentifier == subject.processIdentifier,
      CGRect(origin: try Self.point(window), size: try Self.size(window)).contains(point)
    else { throw NSError(domain: "NativeGestureForeground", code: 1) }
    var hit: AXUIElement?
    guard
      AXUIElementCopyElementAtPosition(
        AXUIElementCreateSystemWide(), Float(point.x), Float(point.y), &hit) == .success,
      let hit
    else { throw NSError(domain: "NativeGestureHitTest", code: 1) }
    var owner: pid_t = 0
    guard AXUIElementGetPid(hit, &owner) == .success, owner == subject.processIdentifier else {
      throw NSError(domain: "NativeGestureHitOwner", code: 1)
    }
    var current: AXUIElement? = hit
    for _ in 0..<12 {
      guard let element = current else { break }
      if CFEqual(element, canvas) { return }
      guard let parent = Self.attribute(element, kAXParentAttribute),
        CFGetTypeID(parent) == AXUIElementGetTypeID()
      else { break }
      current = (parent as! AXUIElement)
    }
    throw NSError(domain: "NativeGestureHitCanvas", code: 1)
  }

  func resize() throws -> CGSize {
    let changed = CGSize(width: originalSize.width - 40, height: originalSize.height - 30)
    try setSize(changed)
    return changed
  }

  func currentSize() throws -> CGSize { try Self.size(window) }

  func deactivate() throws { try focusHelper.activate() }

  func dismissFocusHelper() throws { try focusHelper.terminate() }

  func cleanup() throws {
    // Always release even if cancellation, an assertion, or a later save fails.
    var failures: [String] = []
    do { try pointer.release() } catch { failures.append("mouse-up: \(error)") }
    do { try setSize(originalSize) } catch { failures.append("size restoration: \(error)") }
    do { try focusHelper.terminate() } catch { failures.append("focus helper: \(error)") }
    if !failures.isEmpty {
      throw NSError(
        domain: "NativeGestureCleanup", code: 1,
        userInfo: [
          NSLocalizedDescriptionKey: failures.joined(separator: "; ")
        ])
    }
  }

  private func setSize(_ requested: CGSize) throws {
    var value = requested
    guard let encoded = AXValueCreate(.cgSize, &value) else {
      throw NSError(domain: "NativeGestureResize", code: 1)
    }
    let result = AXUIElementSetAttributeValue(window, kAXSizeAttribute as CFString, encoded)
    guard result == .success else {
      throw NSError(domain: "NativeGestureResize", code: Int(result.rawValue))
    }
  }

  private static func attribute(_ element: AXUIElement, _ name: String) -> CFTypeRef? {
    var result: CFTypeRef?
    guard AXUIElementCopyAttributeValue(element, name as CFString, &result) == .success else {
      return nil
    }
    return result
  }

  private static func findCanvas(_ element: AXUIElement, depth: Int = 0) -> AXUIElement? {
    if attribute(element, kAXIdentifierAttribute) as? String == "sketch.canvas" { return element }
    guard depth < 12 else { return nil }
    for child in attribute(element, kAXChildrenAttribute) as? [AXUIElement] ?? [] {
      if let found = findCanvas(child, depth: depth + 1) { return found }
    }
    return nil
  }

  private static func size(_ element: AXUIElement) throws -> CGSize {
    let raw = try XCTUnwrap(attribute(element, kAXSizeAttribute))
    XCTAssertEqual(CFGetTypeID(raw), AXValueGetTypeID())
    var result = CGSize.zero
    XCTAssertTrue(AXValueGetValue(raw as! AXValue, .cgSize, &result))
    return result
  }

  private static func point(_ element: AXUIElement) throws -> CGPoint {
    let raw = try XCTUnwrap(attribute(element, kAXPositionAttribute))
    XCTAssertEqual(CFGetTypeID(raw), AXValueGetTypeID())
    var result = CGPoint.zero
    XCTAssertTrue(AXValueGetValue(raw as! AXValue, .cgPoint, &result))
    return result
  }
}
