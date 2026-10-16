import XCTest

@MainActor final class AsyncSaveAsTests: XCTestCase {
  private var fixture: NativeSessionFixture!
  override func setUp() async throws {
    continueAfterFailure = false
    fixture = try NativeSessionFixture()
  }
  override func tearDown() async throws { fixture?.record(in: self) }

  private func boundDocument(stage: Int, failure: Bool = false) throws -> URL {
    let f = fixture!
    try f.launch()
    f.draw(count: 1)
    let original = f.document("original")
    try saveDocument(in: f.app, to: original, saveAs: false)
    f.app.terminate()
    try f.launch(stage: stage, failure: failure)
    try openDocument(in: f.app, at: original)
    f.strokes(1)
    return original
  }
  func testPendingSaveAsRetainsOldBindingUntilAcknowledgement() throws {
    let f = fixture!
    let original = try boundDocument(stage: 5)
    let before = try Data(contentsOf: original)
    f.draw(count: 2, y: 0.65)
    let destination = f.document("new-binding")
    try saveDocument(in: f.app, to: destination, saveAs: true, waitForWrite: false)
    try f.reached()
    XCTAssertTrue(f.app.windows.firstMatch.title.contains(original.lastPathComponent))
    f.app.menuBars.menuBarItems["File"].click()
    XCTAssertFalse(f.app.menuItems["Save As…"].isEnabled)
    f.app.typeKey(.escape, modifierFlags: [])
    try f.release()
    f.diskStrokes(2, at: destination)
    f.wait { f.app.windows.firstMatch.title.contains(destination.lastPathComponent) }
    f.app.typeKey("z", modifierFlags: .command)
    f.strokes(1)
    f.app.typeKey("s", modifierFlags: .command)
    f.diskStrokes(1, at: destination)
    XCTAssertEqual(try Data(contentsOf: original), before)
  }
  func testFailedSaveAsRetainsOldFileAndRetryUsesFreshChooser() throws {
    let f = fixture!
    let original = try boundDocument(stage: 5, failure: true)
    let before = try Data(contentsOf: original)
    f.draw(count: 2, y: 0.65)
    let failed = f.document("failed-destination")
    try saveDocument(in: f.app, to: failed, saveAs: true, waitForWrite: false)
    try f.reached()
    let retry = f.app.buttons["sketch.failure.retry"]
    XCTAssertTrue(retry.waitForExistence(timeout: 10))
    XCTAssertTrue(f.app.windows.firstMatch.title.contains(original.lastPathComponent))
    XCTAssertEqual(try Data(contentsOf: original), before)
    XCTAssertFalse(FileManager.default.fileExists(atPath: failed.path))
    retry.click()
    let retryDestination = f.document("fresh-retry")
    try completeNativeSave(in: f.app, to: retryDestination)
    f.diskStrokes(2, at: retryDestination)
    XCTAssertFalse(FileManager.default.fileExists(atPath: failed.path))
    XCTAssertEqual(try Data(contentsOf: original), before)
  }
  func testDirectorySyncWarningReportsWrittenData() throws {
    let f = fixture!
    try f.launch(stage: 7, failure: true)
    f.draw(count: 1)
    let destination = f.document("written-warning")
    try saveDocument(in: f.app, to: destination, saveAs: false, waitForWrite: false)
    try f.reached()
    f.diskStrokes(1, at: destination)
    XCTAssertTrue(
      f.app.staticTexts["The file was written, but durability is unconfirmed"]
        .waitForExistence(timeout: 10))
    f.app.buttons["sketch.failure.cancel"].click()
    f.strokes(1)
  }
}
