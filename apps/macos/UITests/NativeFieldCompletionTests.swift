import XCTest

@MainActor final class NativeFieldCompletionTests: XCTestCase {
  private var fixture: NativeSessionFixture!
  override func setUp() async throws {
    continueAfterFailure = false
    fixture = try NativeSessionFixture()
  }
  override func tearDown() async throws { fixture?.record(in: self) }

  func testSaveCompletionPreservesExistingNonzeroFieldSelection() throws {
    let f = fixture!
    try f.launch(stage: 5)
    f.draw(count: 1)
    let destination = f.document("selected-field-completion")
    try saveDocument(in: f.app, to: destination, saveAs: false, waitForWrite: false)
    try f.reached()
    f.app.buttons["Style and Selection"].click()
    let width = f.app.textFields["sketch.brush.width"]
    width.click()
    width.typeKey("a", modifierFlags: .command)
    width.typeText("12")
    width.typeKey(.leftArrow, modifierFlags: .shift)
    try f.release()
    f.diskStrokes(1, at: destination)
    f.wait { (f.canvas.value as? String ?? "").contains("Storage idle=true") }
    XCTAssertTrue((f.canvas.value as? String ?? "").contains("field=true"))
    XCTAssertEqual(width.value as? String, "12")
    f.app.typeText("8")
    XCTAssertEqual(width.value as? String, "18", "Typing must replace the selected final digit")
    width.typeKey(.return, modifierFlags: [])
    f.app.buttons["Style and Selection"].click()
    f.app.typeKey("z", modifierFlags: .command)
    f.strokes(0)
    f.draw(count: 1)
    f.app.typeKey("s", modifierFlags: .command)
    f.wait { (f.canvas.value as? String ?? "").contains("Storage idle=true,dirty=false") }
    let bytes = try Data(contentsOf: destination)
    XCTAssertGreaterThanOrEqual(bytes.count, 80)
    let bits = bytes[72..<80].enumerated().reduce(UInt64(0)) {
      $0 | UInt64($1.element) << (8 * $1.offset)
    }
    XCTAssertEqual(Double(bitPattern: bits), 18)
  }
}
