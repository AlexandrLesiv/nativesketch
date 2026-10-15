import XCTest

@MainActor final class NativeFocusRestorationTests: XCTestCase {
  func testCancelledSaveAsRestoresNativeWidthSelection() throws {
    let fixture = try NativeSessionFixture()
    defer { fixture.record(in: self) }
    try fixture.launch()
    fixture.draw(count: 1)
    let app = fixture.app
    app.buttons["Style and Selection"].click()
    let width = app.textFields["sketch.brush.width"]
    width.click()
    width.typeKey("a", modifierFlags: .command)
    width.typeText("12")
    width.typeKey(.leftArrow, modifierFlags: .shift)
    app.typeKey("s", modifierFlags: [.command, .shift])
    let panel = nativeFilePanel(in: app)
    XCTAssertTrue(panel.waitForExistence(timeout: 5))
    panel.buttons["Cancel"].click()
    XCTAssertTrue(width.waitForExistence(timeout: 5))
    fixture.wait { (fixture.canvas.value as? String ?? "").contains("field=true") }
    XCTAssertEqual(width.value as? String, "12")
    // Replacing the selected final digit proves native focus and selection restoration.
    app.typeKey("8", modifierFlags: [])
    XCTAssertEqual(width.value as? String, "18")
    width.typeKey(.return, modifierFlags: [])
    app.buttons["Style and Selection"].click()
    fixture.strokes(1)
    let document = fixture.document("focus-preserved")
    try saveDocument(in: app, to: document, saveAs: true)
    fixture.diskStrokes(1, at: document)
  }
}
