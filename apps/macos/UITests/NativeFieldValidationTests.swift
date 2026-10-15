import AppKit
import XCTest

@MainActor final class NativeFieldValidationTests: XCTestCase {
  func testIncompleteWidthRefusesInkAndEscapeRestoresValue() throws {
    let environment = ProcessInfo.processInfo.environment
    let fixture = try NativeSessionFixture()
    fixture.registerTeardown(in: self)
    let app = fixture.app
    let evidence = NativeTestTeardown(test: self)
    evidence.add {
      if app.state != .notRunning {
        evidence.attach(app.screenshot(), name: "final-native-width-validation")
      }
    }
    try fixture.launch()
    let canvas = app.groups["sketch.canvas"]
    XCTAssertTrue(canvas.waitForExistence(timeout: 15))
    app.buttons["Style and Selection"].click()
    let width = app.textFields["sketch.brush.width"]
    XCTAssertTrue(width.waitForExistence(timeout: 5))
    width.click()
    width.typeKey("a", modifierFlags: .command)
    width.typeKey(.delete, modifierFlags: [])
    XCTAssertEqual(width.value as? String, "", "Exercise a real accepted incomplete edit")
    let start = canvas.coordinate(withNormalizedOffset: CGVector(dx: 0.3, dy: 0.4))
    let end = canvas.coordinate(withNormalizedOffset: CGVector(dx: 0.7, dy: 0.6))
    start.click(forDuration: 0.1, thenDragTo: end)
    XCTAssertTrue(app.popovers.firstMatch.exists)
    XCTAssertEqual(width.value as? String, "")
    XCTAssertTrue((canvas.value as? String ?? "").contains("field=true"))
    XCTAssertTrue((canvas.value as? String ?? "").hasPrefix("0 strokes,"))
    let status = app.staticTexts["Document status"]
    XCTAssertTrue((status.value as? String ?? "").contains("Enter a stroke width"))
    let invalid = XCTAttachment(screenshot: app.screenshot())
    invalid.name = "incomplete-width-retains-native-edit"
    invalid.lifetime = .keepAlways
    add(invalid)
    width.typeKey(.escape, modifierFlags: [])
    XCTAssertEqual(width.value as? String, "6")
    XCTAssertFalse((status.value as? String ?? "").contains("Enter a stroke width"))
    width.typeKey("a", modifierFlags: .command)
    width.typeKey(.delete, modifierFlags: [])
    width.typeKey(.return, modifierFlags: [])
    XCTAssertTrue((status.value as? String ?? "").contains("Enter a stroke width"))
    width.typeText("12")
    width.typeKey(.return, modifierFlags: [])
    XCTAssertFalse((status.value as? String ?? "").contains("Enter a stroke width"))
    app.buttons["Style and Selection"].click()
    XCTAssertTrue(app.popovers.firstMatch.waitForNonExistence(timeout: 5))
    start.click(forDuration: 0.1, thenDragTo: end)
    let committed = expectation(
      for: NSPredicate(format: "value BEGINSWITH '1 strokes,'"), evaluatedWith: canvas)
    wait(for: [committed], timeout: 5)
    let root = URL(fileURLWithPath: try XCTUnwrap(environment["NATIVE_SKETCH_DOCUMENT"]))
      .deletingLastPathComponent()
    let destination = root.appendingPathComponent("validated-width-\(UUID().uuidString).nsketch")
    XCTAssertFalse(FileManager.default.fileExists(atPath: destination.path))
    try saveDocument(in: app, to: destination, saveAs: false)
    let data = try Data(contentsOf: destination)
    guard data.count >= 80 else {
      XCTFail("Missing serialized stroke width")
      return
    }
    let widthBits = data[72..<80].enumerated().reduce(UInt64(0)) {
      $0 | (UInt64($1.element) << ($1.offset * 8))
    }
    XCTAssertEqual(Double(bitPattern: widthBits), 12)
    app.typeKey("z", modifierFlags: .command)
    let undone = expectation(
      for: NSPredicate(format: "value BEGINSWITH '0 strokes,'"), evaluatedWith: canvas)
    wait(for: [undone], timeout: 5)
    app.menuBars.menuBarItems["Edit"].click()
    XCTAssertFalse(app.menuItems["Undo"].isEnabled, "Refused ink must not add a history operation")
    app.typeKey(.escape, modifierFlags: [])
  }
}
