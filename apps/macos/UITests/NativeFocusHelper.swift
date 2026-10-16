import AppKit
import CryptoKit
import XCTest

/// A separate disposable native application gives XCUIAutomation an explicit activation target.
@MainActor final class NativeFocusHelper {
  private let app: XCUIApplication
  private let runtimeRoot: URL
  private let bundleURL: URL
  private let bundleID: String
  private let expectedHash: String
  private var launchAttempted = false

  init(fixture: NativeSessionFixture, subject: NSRunningApplication) throws {
    guard let rawPath = ProcessInfo.processInfo.environment["NATIVE_SKETCH_FOCUS_APP"],
      rawPath.hasPrefix("/"), let subjectURL = subject.bundleURL,
      let subjectID = subject.bundleIdentifier,
      let hash = ProcessInfo.processInfo.environment["NATIVE_SKETCH_FOCUS_BINARY_SHA256"],
      hash.count == 64, hash.allSatisfy({ "0123456789abcdef".contains($0) })
    else {
      throw Self.failure("Missing paired focus-helper identity")
    }
    let supplied = URL(fileURLWithPath: rawPath).resolvingSymlinksInPath()
    let expected = subjectURL.deletingLastPathComponent()
      .appendingPathComponent("FocusSubject.app").resolvingSymlinksInPath()
    guard supplied.path == expected.path,
      Bundle(url: supplied)?.bundleIdentifier == subjectID + ".focus"
    else {
      throw Self.failure("Focus helper must be the exact paired disposable app")
    }
    runtimeRoot = fixture.root.appendingPathComponent("focus-runtime", isDirectory: true)
    bundleURL = supplied
    bundleID = subjectID + ".focus"
    expectedHash = hash
    app = XCUIApplication(url: supplied)
  }

  func activate() throws {
    try NativeTestProvenance.verify(for: NativeSessionFixture.self)
    guard let executable = Bundle(url: bundleURL)?.executableURL else {
      throw Self.failure("Paired helper executable is missing")
    }
    let bytes = try Data(contentsOf: executable, options: .mappedIfSafe)
    let actualHash = SHA256.hash(data: bytes).map { String(format: "%02x", $0) }.joined()
    guard actualHash == expectedHash else {
      throw Self.failure("Paired helper binary hash changed")
    }
    app.launchArguments = [
      "-AppleLanguages", "(en)", "-AppleLocale", "en_US",
      "--runtime-root", runtimeRoot.path,
    ]
    let identity = try JSONSerialization.data(
      withJSONObject: [
        "schemaVersion": 1, "bundlePath": bundleURL.path, "bundleIdentifier": bundleID,
        "binarySha256": actualHash, "runtimeRoot": runtimeRoot.path,
      ], options: [.sortedKeys])
    XCTContext.runActivity(named: "Paired native focus helper") { activity in
      let attachment = XCTAttachment(string: String(decoding: identity, as: UTF8.self))
      attachment.name = "native-focus-helper-identity"
      attachment.lifetime = .keepAlways
      activity.add(attachment)
    }
    guard NSRunningApplication.runningApplications(withBundleIdentifier: bundleID).isEmpty else {
      throw Self.failure("Refusing to launch over a pre-existing same-ID helper")
    }
    launchAttempted = true
    app.launch()
    XCTAssertTrue(
      app.wait(for: .runningForeground, timeout: 10), "Focus helper must actually activate")
    guard let foreground = NSWorkspace.shared.frontmostApplication,
      foreground.bundleIdentifier == bundleID,
      foreground.bundleURL?.resolvingSymlinksInPath().path == bundleURL.path
    else {
      throw Self.failure("Foreground helper is not the current paired bundle")
    }
    XCTContext.runActivity(named: "Actual paired foreground process") { activity in
      let attachment = XCTAttachment(
        string:
          "pid=\(foreground.processIdentifier), bundle=\(bundleID), path=\(bundleURL.path)")
      attachment.name = "native-focus-helper-foreground"
      attachment.lifetime = .keepAlways
      activity.add(attachment)
    }
  }

  func terminate() throws {
    guard launchAttempted else { return }
    let running = NSRunningApplication.runningApplications(withBundleIdentifier: bundleID)
    guard running.allSatisfy({ $0.bundleURL?.resolvingSymlinksInPath().path == bundleURL.path })
    else {
      throw Self.failure("Refusing to terminate a helper from another attempt")
    }
    if !running.isEmpty { app.terminate() }
    launchAttempted = false
  }

  private static func failure(_ message: String) -> NSError {
    NSError(
      domain: "NativeFocusHelperIdentity", code: 1, userInfo: [NSLocalizedDescriptionKey: message])
  }
}
