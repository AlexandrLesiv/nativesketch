import XCTest

@MainActor final class NativeInspectorDismissalTests: XCTestCase {
  func testSmallDarkWindowFitsInspectorAndAcceptsDrawing() throws {
    continueAfterFailure = false
    let fixture = try NativeSessionFixture()
    fixture.registerTeardown(in: self)
    try fixture.launch(extraArguments: ["--small-window", "--appearance", "dark"])
    let app = fixture.app
    let window = app.windows["Untitled — Native Sketch"]
    XCTAssertLessThanOrEqual(window.frame.width, 700)
    XCTAssertLessThanOrEqual(window.frame.height, 600)
    app.buttons["Style and Selection"].click()
    let inspector = app.popovers.firstMatch
    XCTAssertTrue(inspector.waitForExistence(timeout: 5))
    for control in [app.textFields["sketch.brush.width"], app.colorWells["sketch.brush.color"]] {
      XCTAssertTrue(control.exists)
      XCTAssertTrue(inspector.frame.contains(control.frame))
    }
    let screenshot = XCTAttachment(screenshot: app.screenshot())
    screenshot.name = "private-small-dark-inspector"
    screenshot.lifetime = .keepAlways
    add(screenshot)
    app.buttons["Style and Selection"].click()
    XCTAssertTrue(inspector.waitForNonExistence(timeout: 5))
    fixture.draw(count: 1)
  }

  func testInspectorEscapeRestoresCanvasKeyboardCommands() throws {
    let fixture = try NativeSessionFixture()
    fixture.registerTeardown(in: self)
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
