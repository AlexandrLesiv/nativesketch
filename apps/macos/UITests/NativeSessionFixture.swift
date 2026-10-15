import XCTest

/// Real native input and disposable filesystem observations. No editor mutation hooks.
@MainActor final class NativeSessionFixture {
  let app: XCUIApplication
  let root: URL
  let runtimeRoot: URL
  private(set) var gate: URL?
  var canvas: XCUIElement { app.groups["sketch.canvas"] }
  init() throws {
    let environment = ProcessInfo.processInfo.environment
    app = XCUIApplication(
      url: URL(fileURLWithPath: try XCTUnwrap(environment["NATIVE_SKETCH_APP"])))
    // Xcode's UI runner is sandboxed even when the test bundle target is not.
    // Use its own writable temporary directory; the unsandboxed subject receives
    // only this explicit disposable root, never the user's document/recovery store.
    root = FileManager.default.temporaryDirectory
      .appendingPathComponent("native-sketch-session-\(UUID().uuidString)")
    runtimeRoot = root.appendingPathComponent("runtime")
    try FileManager.default.createDirectory(at: root, withIntermediateDirectories: true)
  }
  func launch(stage: Int? = nil, failure: Bool = false, extraArguments: [String] = []) throws {
    try NativeTestProvenance.verify(for: NativeSessionFixture.self)
    app.launchArguments = [
      "-AppleLanguages", "(en)", "-AppleLocale", "en_US",
      "--native-ui-observation", "--runtime-root", runtimeRoot.path,
    ]
    app.launchArguments += extraArguments
    if let stage {
      let control = root.appendingPathComponent("gate-\(UUID().uuidString)")
      try FileManager.default.createDirectory(at: control, withIntermediateDirectories: false)
      gate = control
      app.launchArguments += [
        "--test-storage-root", control.path,
        "--test-storage-stage", String(stage), "--test-storage-mode", failure ? "2" : "1",
      ]
    } else {
      gate = nil
    }
    app.launch()
    XCTAssertTrue(canvas.waitForExistence(timeout: 15))
    wait { (self.canvas.value as? String ?? "").contains("active=true,key=true,canvas=true") }
  }
  func document(_ name: String) -> URL { root.appendingPathComponent(name + ".nsketch") }
  func draw(count: Int, y: Double = 0.4) {
    canvas.coordinate(withNormalizedOffset: CGVector(dx: 0.3, dy: y)).click(
      forDuration: 0.1,
      thenDragTo: canvas.coordinate(withNormalizedOffset: CGVector(dx: 0.7, dy: y + 0.15)))
    strokes(count)
  }
  func strokes(_ count: Int) {
    wait { (self.canvas.value as? String ?? "").hasPrefix("\(count) strokes,") }
  }
  func wait(
    _ predicate: @escaping () -> Bool, timeout: TimeInterval = 15,
    file: StaticString = #filePath, line: UInt = #line
  ) {
    let expectation = XCTNSPredicateExpectation(
      predicate: NSPredicate { _, _ in predicate() }, object: nil)
    XCTAssertEqual(
      XCTWaiter.wait(for: [expectation], timeout: timeout), .completed, file: file, line: line)
  }
  func reached() throws {
    let marker = try XCTUnwrap(gate).appendingPathComponent("reached")
    wait { FileManager.default.fileExists(atPath: marker.path) }
  }
  func release() throws {
    try Data().write(to: try XCTUnwrap(gate).appendingPathComponent("release"))
  }
  func diskStrokes(_ count: UInt64, at url: URL) {
    wait {
      guard let bytes = try? Data(contentsOf: url), bytes.count >= 16,
        String(data: bytes.prefix(8), encoding: .utf8) == "NSKETCH1"
      else { return false }
      let actual = bytes[8..<16].enumerated().reduce(UInt64(0)) {
        $0 | UInt64($1.element) << (8 * $1.offset)
      }
      return actual == count
    }
  }
  func record(in test: XCTestCase) {
    let evidence = NativeTestTeardown(test: test)
    var failure: Error?
    do { try captureAndRelease(in: evidence) } catch { failure = error }
    app.terminate()
    if let failure { XCTFail("Native fixture cleanup failed: \(failure)") }
  }

  /// Register before launch. Termination is a separate last block so evidence/worker-gate
  /// failures cannot prevent it; gesture cleanup registered later runs first.
  func registerTeardown(in test: XCTestCase) {
    let evidence = NativeTestTeardown(test: test)
    evidence.add { self.app.terminate() }
    evidence.add { try self.captureAndRelease(in: evidence) }
  }

  private func captureAndRelease(in evidence: NativeTestTeardown) throws {
    evidence.attach(root.path, name: "native-session-fixture-path")
    if let gate {
      do { try Data().write(to: gate.appendingPathComponent("release")) } catch {
        evidence.attach(String(describing: error), name: "native-session-gate-cleanup-error")
        throw error
      }
    }
    evidence.attach(app.debugDescription, name: "native-session-final-accessibility")
  }
}
