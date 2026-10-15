import XCTest

@MainActor final class NativeRoutingTests: XCTestCase {
  private var fixture: NativeSessionFixture!
  override func setUp() async throws {
    continueAfterFailure = false
    fixture = try NativeSessionFixture()
  }
  override func tearDown() async throws { fixture?.record(in: self) }
  func testMenusShortcutsAndControlsShareEditingState() throws {
    let f = fixture!
    try f.launch()
    f.app.typeKey("v", modifierFlags: [])
    XCTAssertEqual(f.app.radioButtons["Select"].value as? NSNumber, NSNumber(value: true))
    f.app.radioButtons["Pen"].click()
    XCTAssertEqual(f.app.radioButtons["Pen"].value as? NSNumber, NSNumber(value: true))
    f.draw(count: 1)
    let original = f.document("routing-original")
    try saveDocument(in: f.app, to: original, saveAs: false)
    let bytes = try Data(contentsOf: original)
    f.app.menuBars.menuBarItems["Edit"].click()
    f.app.menuItems["Undo"].click()
    f.strokes(0)
    f.app.typeKey("z", modifierFlags: [.command, .shift])
    f.strokes(1)
    f.app.menuBars.menuBarItems["Tools"].click()
    f.app.menuItems["Select"].click()
    f.app.menuBars.menuBarItems["Edit"].click()
    f.app.menuItems["Select Next Stroke"].click()
    XCTAssertTrue((f.canvas.value as? String ?? "").contains("one selected"))
    f.app.typeKey(.delete, modifierFlags: [])
    f.strokes(0)
    f.app.typeKey("z", modifierFlags: .command)
    f.strokes(1)
    let restored = f.document("routing-restored")
    try saveDocument(in: f.app, to: restored, saveAs: true)
    XCTAssertEqual(try Data(contentsOf: restored), bytes)
  }

  func testRealWriteFailureWaitsBehindRecoverySheet() throws {
    let f = fixture!
    // Hold before staging-file creation, then remove only this empty disposable directory.
    try f.launch(stage: 2)
    f.draw(count: 1)
    let directory = f.root.appendingPathComponent("removed-destination")
    try FileManager.default.createDirectory(at: directory, withIntermediateDirectories: false)
    let destination = directory.appendingPathComponent("drawing.nsketch")
    try saveDocument(in: f.app, to: destination, saveAs: false, waitForWrite: false)
    try f.reached()
    f.app.menuBars.menuBarItems["File"].click()
    f.app.menuItems["Recover Drawings…"].click()
    XCTAssertTrue(f.app.tables["sketch.recovery.list"].waitForExistence(timeout: 5))
    try FileManager.default.removeItem(at: directory)
    try f.release()
    f.wait { !f.app.staticTexts["sketch.recovery.status"].label.contains("Loading") }
    XCTAssertFalse(
      f.app.buttons["sketch.failure.cancel"].exists,
      "A save failure must not stack a sheet over recovery")
    f.app.buttons["sketch.recovery.later"].click()
    let cancel = f.app.buttons["sketch.failure.cancel"]
    XCTAssertTrue(cancel.waitForExistence(timeout: 10), "The deferred failure must remain visible")
    cancel.click()
    f.strokes(1)
    XCTAssertFalse(FileManager.default.fileExists(atPath: destination.path))
    let fresh = f.document("after-real-failure")
    try saveDocument(in: f.app, to: fresh, saveAs: false)
    f.diskStrokes(1, at: fresh)
  }
}
