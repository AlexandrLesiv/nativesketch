import XCTest

@MainActor
func chooseBrush(in app: XCUIApplication, width value: String, hex: String) throws {
  app.buttons["Style and Selection"].click()
  let width = app.textFields["sketch.brush.width"]
  XCTAssertTrue(width.waitForExistence(timeout: 5))
  width.click()
  width.typeKey("a", modifierFlags: .command)
  width.typeText(value)
  width.typeKey(.return, modifierFlags: [])
  app.colorWells["sketch.brush.color"].click()
  let colors = app.windows["Colors"]
  XCTAssertTrue(colors.waitForExistence(timeout: 5))
  colors.buttons["Color Sliders"].click()
  colors.popUpButtons.firstMatch.click()
  app.menuItems["RGB Sliders"].click()
  let hexadecimal = colors.textFields["hex"]
  XCTAssertTrue(hexadecimal.waitForExistence(timeout: 5))
  hexadecimal.click()
  hexadecimal.typeKey("a", modifierFlags: .command)
  hexadecimal.typeText(hex)
  hexadecimal.typeKey(.return, modifierFlags: [])
  colors.buttons[XCUIIdentifierCloseWindow].click()
  XCTAssertTrue(colors.waitForNonExistence(timeout: 5))
  if app.popovers.firstMatch.exists {
    app.buttons["Style and Selection"].click()
  }
  XCTAssertTrue(app.popovers.firstMatch.waitForNonExistence(timeout: 5))
}

@MainActor
func saveDocument(in app: XCUIApplication, to url: URL, saveAs: Bool) throws {
  app.typeKey("s", modifierFlags: saveAs ? [.command, .shift] : .command)
  let panel = app.dialogs.firstMatch
  XCTAssertTrue(panel.waitForExistence(timeout: 5))
  let name = panel.textFields["saveAsNameTextField"]
  XCTAssertTrue(name.waitForExistence(timeout: 5))
  name.click()
  name.typeKey("a", modifierFlags: .command)
  name.typeText(url.lastPathComponent)
  try navigateFilePanel(app, to: url.deletingLastPathComponent())
  XCTAssertTrue(panel.buttons["Save"].isEnabled)
  panel.buttons["Save"].click()
  let saved = XCTNSPredicateExpectation(
    predicate: NSPredicate { _, _ in
      FileManager.default.fileExists(atPath: url.path)
    }, object: nil)
  XCTAssertEqual(XCTWaiter.wait(for: [saved], timeout: 10), .completed)
  XCTAssertTrue(panel.waitForNonExistence(timeout: 5))
}

@MainActor
func openDocument(in app: XCUIApplication, at url: URL) throws {
  app.typeKey("o", modifierFlags: .command)
  let panel = app.dialogs.firstMatch
  XCTAssertTrue(panel.waitForExistence(timeout: 5))
  try navigateFilePanel(app, to: url)
  panel.buttons["Open"].click()
  XCTAssertTrue(panel.waitForNonExistence(timeout: 5))
}

@MainActor
private func navigateFilePanel(_ app: XCUIApplication, to url: URL) throws {
  app.typeKey("g", modifierFlags: [.command, .shift])
  let sheet = app.sheets["GoToWindow"]
  XCTAssertTrue(sheet.waitForExistence(timeout: 5))
  let field = sheet.textFields["PathTextField"]
  XCTAssertTrue(field.waitForExistence(timeout: 5))
  field.typeKey("a", modifierFlags: .command)
  field.typeText(url.path)
  field.typeKey(.return, modifierFlags: [])
  XCTAssertTrue(sheet.waitForNonExistence(timeout: 5))
}
