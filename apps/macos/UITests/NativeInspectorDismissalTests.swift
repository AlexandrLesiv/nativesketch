import XCTest

@MainActor final class NativeInspectorDismissalTests: XCTestCase {
  func testInspectorEscapeRestoresCanvasKeyboardCommands() throws {
    let fixture = try NativeSessionFixture()
    defer { fixture.record(in: self) }
    try fixture.launch()
    fixture.draw(count: 1)
    let app = fixture.app
    app.buttons["Style and Selection"].click()
    let width = app.textFields["sketch.brush.width"]
    XCTAssertTrue(width.waitForExistence(timeout: 5))
    width.click()
    width.typeKey("a", modifierFlags: .command)
    width.typeText("12")
    width.typeKey(.return, modifierFlags: [])
    app.typeKey(.escape, modifierFlags: [])
    XCTAssertTrue(width.waitForNonExistence(timeout: 5))
    fixture.wait { (fixture.canvas.value as? String ?? "").contains("canvas=true") }
    app.typeKey("v", modifierFlags: [])
    XCTAssertEqual(app.radioButtons["Select"].value as? NSNumber, NSNumber(value: true))
    app.typeKey("p", modifierFlags: [])
    XCTAssertEqual(app.radioButtons["Pen"].value as? NSNumber, NSNumber(value: true))
    fixture.draw(count: 2, y: 0.65)
    let destination = fixture.document("inspector-dismissal")
    try saveDocument(in: app, to: destination, saveAs: false)
    fixture.diskStrokes(2, at: destination)
  }
}
