import XCTest

@MainActor final class NativePoisonClosureTests: XCTestCase {
  func testCaughtEnginePanicAllowsCancelledCloseAndQuitThenSafeExit() throws {
    continueAfterFailure = false
    let f = try NativeSessionFixture()
    f.registerTeardown(in: self)
    try f.launch(extraArguments: ["--test-poison-on-close"])
    f.draw(count: 1)
    let original = f.document("last-good-before-panic")
    try saveDocument(in: f.app, to: original, saveAs: false)
    f.diskStrokes(1, at: original)
    f.wait { (f.canvas.value as? String ?? "").contains("Storage idle=true,dirty=false") }
    let bytes = try Data(contentsOf: original)
    f.draw(count: 2, y: 0.65)

    // This native close triggers an explicit test-only panic inside the real C boundary.
    // Swift receives PANIC normally; no editor state or success result is injected.
    f.app.typeKey("w", modifierFlags: .command)
    let cancel = f.app.buttons["sketch.unavailable.cancel"]
    XCTAssertTrue(cancel.waitForExistence(timeout: 5))
    cancel.click()
    XCTAssertTrue(cancel.waitForNonExistence(timeout: 5))
    XCTAssertTrue(f.canvas.exists)
    XCTAssertNotEqual(f.app.state, .notRunning)
    XCTAssertEqual(try Data(contentsOf: original), bytes)

    f.app.typeKey("q", modifierFlags: .command)
    XCTAssertTrue(cancel.waitForExistence(timeout: 5))
    cancel.click()
    XCTAssertTrue(cancel.waitForNonExistence(timeout: 5))
    XCTAssertNotEqual(f.app.state, .notRunning)
    f.app.typeKey("q", modifierFlags: .command)
    let close = f.app.buttons["sketch.unavailable.close"]
    XCTAssertTrue(close.waitForExistence(timeout: 5))
    close.click()
    f.wait { f.app.state == .notRunning }
    XCTAssertEqual(try Data(contentsOf: original), bytes)
    try f.launch()
    try openDocument(in: f.app, at: original)
    f.strokes(1)
    XCTAssertEqual(try Data(contentsOf: original), bytes)
  }
}
