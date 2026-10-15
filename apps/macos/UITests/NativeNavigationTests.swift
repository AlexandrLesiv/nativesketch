import XCTest

@MainActor final class NativeNavigationTests: XCTestCase {
  private var fixture: NativeSessionFixture!
  override func setUp() async throws {
    continueAfterFailure = false
    fixture = try NativeSessionFixture()
  }
  override func tearDown() async throws { fixture?.record(in: self) }

  func testNewCancelAndDiscardPreservePendingSaveSemantics() throws {
    let f = fixture!
    try f.launch(stage: 5)
    f.draw(count: 1)
    let destination = f.document("accepted-save")
    try saveDocument(in: f.app, to: destination, saveAs: false, waitForWrite: false)
    try f.reached()
    f.draw(count: 2, y: 0.65)
    f.app.typeKey("n", modifierFlags: .command)
    let cancel = f.app.buttons["sketch.navigation.cancel"]
    XCTAssertTrue(cancel.waitForExistence(timeout: 5))
    cancel.click()
    f.strokes(2)
    try f.release()
    f.diskStrokes(1, at: destination)
    f.wait { (f.canvas.value as? String ?? "").contains("Storage idle=true,dirty=true") }
    f.app.typeKey("n", modifierFlags: .command)
    let prompt = nativeFilePanel(in: f.app)
    prompt.buttons["Cancel"].click()
    f.strokes(2)
    f.app.typeKey("n", modifierFlags: .command)
    nativeFilePanel(in: f.app).buttons["Don’t Save"].click()
    f.strokes(0)
    f.diskStrokes(1, at: destination)
    XCTAssertTrue(f.app.windows.firstMatch.title.contains("Untitled"))
  }

  func testQuitCancellationAndSuccessfulDrain() throws {
    let f = fixture!
    try f.launch(stage: 5)
    f.draw(count: 1)
    let destination = f.document("quit-save")
    try saveDocument(in: f.app, to: destination, saveAs: false, waitForWrite: false)
    try f.reached()
    f.app.typeKey("q", modifierFlags: .command)
    XCTAssertNotEqual(f.app.state, .notRunning)
    let cancel = f.app.buttons["sketch.navigation.cancel"]
    XCTAssertTrue(cancel.waitForExistence(timeout: 5))
    f.app.typeKey(.escape, modifierFlags: [])
    XCTAssertTrue(cancel.waitForNonExistence(timeout: 5))
    XCTAssertNotEqual(f.app.state, .notRunning)
    f.app.typeKey("q", modifierFlags: .command)
    XCTAssertTrue(cancel.waitForExistence(timeout: 5))
    cancel.click()
    try f.release()
    f.diskStrokes(1, at: destination)
    f.wait { (f.canvas.value as? String ?? "").contains("Storage idle=true") }
    f.draw(count: 2, y: 0.65)
    f.app.typeKey("q", modifierFlags: .command)
    nativeFilePanel(in: f.app).buttons["Save"].click()
    f.wait { f.app.state == .notRunning }
    f.diskStrokes(2, at: destination)
    try f.launch()
    try openDocument(in: f.app, at: destination)
    f.strokes(2)
  }

  func testInvalidWidthBlocksQuitUntilEscapeRestoration() throws {
    let f = fixture!
    try f.launch()
    f.draw(count: 1)
    f.app.buttons["Style and Selection"].click()
    let width = f.app.textFields["sketch.brush.width"]
    width.click()
    width.typeKey("a", modifierFlags: .command)
    width.typeKey(.delete, modifierFlags: [])
    f.app.typeKey("q", modifierFlags: .command)
    XCTAssertNotEqual(f.app.state, .notRunning)
    XCTAssertEqual(width.value as? String, "")
    XCTAssertTrue((f.canvas.value as? String ?? "").contains("field=true"))
    XCTAssertFalse(f.app.sheets.firstMatch.exists)
    width.typeKey(.escape, modifierFlags: [])
    XCTAssertEqual(width.value as? String, "6")
    width.typeKey(.return, modifierFlags: [])
    f.app.buttons["Style and Selection"].click()
    f.app.typeKey("q", modifierFlags: .command)
    nativeFilePanel(in: f.app).buttons["Cancel"].click()
    f.strokes(1)
  }
}
