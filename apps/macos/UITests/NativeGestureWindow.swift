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
  private var focusWindow: NSWindow?
  private var originalActivationPolicy: NSApplication.ActivationPolicy?

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
      windowBounds: CGRect(origin: Self.point(window), size: Self.size(window)))
  }

  func resize() throws -> CGSize {
    let changed = CGSize(width: originalSize.width - 40, height: originalSize.height - 30)
    try setSize(changed)
    return changed
  }

  func currentSize() throws -> CGSize { try Self.size(window) }

  func deactivate() {
    let helper = NSApplication.shared
    originalActivationPolicy = helper.activationPolicy()
    helper.setActivationPolicy(.regular)
    let owned = NSWindow(
      contentRect: NSRect(x: 40, y: 40, width: 260, height: 100),
      styleMask: [.titled], backing: .buffered, defer: false)
    owned.isReleasedWhenClosed = false
    owned.title = "Native Sketch disposable gesture test"
    focusWindow = owned
    owned.makeKeyAndOrderFront(nil)
    helper.activate(ignoringOtherApps: true)
  }

  func hideFocusWindow() { focusWindow?.orderOut(nil) }

  func cleanup() throws {
    // Always release even if cancellation, an assertion, or a later save fails.
    var failures: [String] = []
    do { try pointer.release() } catch { failures.append("mouse-up: \(error)") }
    do { try setSize(originalSize) } catch { failures.append("size restoration: \(error)") }
    focusWindow?.close()
    focusWindow = nil
    if let originalActivationPolicy {
      if !NSApplication.shared.setActivationPolicy(originalActivationPolicy) {
        failures.append("activation policy restoration rejected")
      }
      self.originalActivationPolicy = nil
    }
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
