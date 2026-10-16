import XCTest

@MainActor final class AsyncSaveTests: XCTestCase {
  private var fixture: NativeSessionFixture!
  override func setUp() async throws {
    continueAfterFailure = false
    fixture = try NativeSessionFixture()
  }
  override func tearDown() async throws { fixture?.record(in: self) }

  func testAdmittedSaveKeepsNewerDrawingDirty() throws {
    let f = fixture!
    // BEFORE_REPLACE holds the real encoded snapshot A, not resolution or checkpoint work.
    try f.launch(stage: 5)
    f.draw(count: 1)
    let destination = f.document("snapshot-a")
    try saveDocument(in: f.app, to: destination, saveAs: false, waitForWrite: false)
    try f.reached()
    f.draw(count: 2, y: 0.65)
    try f.release()
    f.diskStrokes(1, at: destination)
    f.wait { f.app.windows.firstMatch.title.contains(destination.lastPathComponent) }
    f.wait { (f.canvas.value as? String ?? "").contains("Storage idle=true,dirty=true") }
    // Saving B must remain enabled and write the second stroke to the confirmed binding.
    f.app.typeKey("s", modifierFlags: .command)
    f.diskStrokes(2, at: destination)
    XCTAssertFalse(f.app.sheets.firstMatch.exists)
  }

  func testSaveCompletionPreservesIncompleteFieldEditing() throws {
    let f = fixture!
    try f.launch(stage: 5)
    f.draw(count: 1)
    let destination = f.document("field-completion")
    try saveDocument(in: f.app, to: destination, saveAs: false, waitForWrite: false)
    try f.reached()
    f.app.buttons["Style and Selection"].click()
    let width = f.app.textFields["sketch.brush.width"]
    width.click()
    width.typeKey("a", modifierFlags: .command)
    width.typeKey(.delete, modifierFlags: [])
    XCTAssertEqual(width.value as? String, "")
    try f.release()
    f.diskStrokes(1, at: destination)
    f.wait { f.app.windows.firstMatch.title.contains(destination.lastPathComponent) }
    XCTAssertEqual(width.value as? String, "")
    XCTAssertTrue((f.canvas.value as? String ?? "").contains("field=true"))
    width.typeKey("z", modifierFlags: .command)
    XCTAssertEqual(width.value as? String, "6")
    width.typeKey("z", modifierFlags: [.command, .shift])
    XCTAssertEqual(width.value as? String, "")
    width.typeText("12")
    width.typeKey(.leftArrow, modifierFlags: [])
    width.typeText("0")
    XCTAssertEqual(width.value as? String, "102", "Native caret insertion remains intact")
    width.typeKey("a", modifierFlags: .command)
    width.typeText("12")
    width.typeKey(.return, modifierFlags: [])
    f.app.buttons["Style and Selection"].click()
    f.app.typeKey("z", modifierFlags: .command)
    f.strokes(0)
    f.draw(count: 1)
    let corrected = f.document("corrected-width")
    try saveDocument(in: f.app, to: corrected, saveAs: true)
    let bytes = try Data(contentsOf: corrected)
    XCTAssertGreaterThanOrEqual(bytes.count, 80)
    let widthBits = bytes[72..<80].enumerated().reduce(UInt64(0)) {
      $0 | UInt64($1.element) << (8 * $1.offset)
    }
    XCTAssertEqual(Double(bitPattern: widthBits), 12)
  }
}
