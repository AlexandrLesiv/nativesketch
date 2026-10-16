import XCTest

@MainActor final class NativeDecisionTests: XCTestCase {
  private var fixture: NativeSessionFixture!
  override func setUp() async throws {
    continueAfterFailure = false
    fixture = try NativeSessionFixture()
  }
  override func tearDown() async throws { fixture?.record(in: self) }

  func testNewSaveWritesCurrentDrawingBeforeReplacingIt() throws {
    let f = fixture!
    try f.launch()
    f.draw(count: 1)
    f.app.typeKey("n", modifierFlags: .command)
    nativeFilePanel(in: f.app).buttons["Save"].click()
    let destination = f.document("before-new")
    try completeNativeSave(in: f.app, to: destination)
    f.strokes(0)
    f.diskStrokes(1, at: destination)
    XCTAssertTrue(f.app.windows.firstMatch.title.contains("Untitled"))
    XCTAssertTrue((f.canvas.value as? String ?? "").contains("dirty=false"))
  }

  func testOpenCancelDiscardAndSaveUseCurrentDirtyDrawing() throws {
    let f = fixture!
    try f.launch()
    f.draw(count: 1)
    let destination = f.document("open-decisions")
    try saveDocument(in: f.app, to: destination, saveAs: false)
    f.diskStrokes(1, at: destination)
    let before = try Data(contentsOf: destination)
    f.draw(count: 2, y: 0.6)
    f.app.typeKey("o", modifierFlags: .command)
    nativeFilePanel(in: f.app).buttons["Cancel"].click()
    f.strokes(2)
    XCTAssertEqual(try Data(contentsOf: destination), before)
    XCTAssertTrue(f.app.windows.firstMatch.title.contains(destination.lastPathComponent))

    f.app.typeKey("o", modifierFlags: .command)
    nativeFilePanel(in: f.app).buttons["Don’t Save"].click()
    try completeNativeOpen(in: f.app, at: destination)
    f.strokes(1)
    XCTAssertEqual(try Data(contentsOf: destination), before)
    f.draw(count: 2, y: 0.6)
    f.app.typeKey("o", modifierFlags: .command)
    nativeFilePanel(in: f.app).buttons["Save"].click()
    try completeNativeOpen(in: f.app, at: destination)
    f.strokes(2)
    f.diskStrokes(2, at: destination)
    f.wait { (f.canvas.value as? String ?? "").contains("Storage idle=true,dirty=false") }
  }

  func testCloseCancellationKeepsDrawingAndSaveClosesWindow() throws {
    let f = fixture!
    try f.launch(stage: 5)
    f.draw(count: 1)
    f.app.typeKey("w", modifierFlags: .command)
    nativeFilePanel(in: f.app).buttons["Cancel"].click()
    f.strokes(1)
    XCTAssertTrue(f.canvas.exists)
    let destination = f.document("before-close")
    try saveDocument(in: f.app, to: destination, saveAs: false, waitForWrite: false)
    try f.reached()
    f.app.typeKey("w", modifierFlags: .command)
    let waiting = f.app.buttons["sketch.navigation.cancel"]
    XCTAssertTrue(waiting.waitForExistence(timeout: 5))
    waiting.click()
    f.draw(count: 2, y: 0.6)
    try f.release()
    f.diskStrokes(1, at: destination)
    f.wait { (f.canvas.value as? String ?? "").contains("Storage idle=true,dirty=true") }
    f.app.typeKey("w", modifierFlags: .command)
    nativeFilePanel(in: f.app).buttons["Save"].click()
    f.diskStrokes(2, at: destination)
    f.wait { !f.canvas.exists }
  }

  func testFailedSaveDuringQuitKeepsAppAndDrawingOpen() throws {
    let f = fixture!
    try f.launch(stage: 5, failure: true)
    f.draw(count: 1)
    f.app.typeKey("q", modifierFlags: .command)
    nativeFilePanel(in: f.app).buttons["Save"].click()
    let failed = f.document("quit-failure")
    try completeNativeSave(in: f.app, to: failed, waitForWrite: false)
    try f.reached()
    let cancel = f.app.buttons["sketch.failure.cancel"]
    XCTAssertTrue(cancel.waitForExistence(timeout: 10))
    XCTAssertNotEqual(f.app.state, .notRunning)
    cancel.click()
    f.strokes(1)
    XCTAssertFalse(FileManager.default.fileExists(atPath: failed.path))
    XCTAssertTrue((f.canvas.value as? String ?? "").contains("dirty=true"))
    f.draw(count: 2, y: 0.6)
    f.app.typeKey("q", modifierFlags: .command)
    nativeFilePanel(in: f.app).buttons["Don’t Save"].click()
    f.wait { f.app.state == .notRunning }
  }
}
