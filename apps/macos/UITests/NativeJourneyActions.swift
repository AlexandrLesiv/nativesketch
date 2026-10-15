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
  let mainWindow = app.windows["Untitled — Native Sketch"]
  // Keep the palette and expanded profile menu over the test-owned canvas for evidence.
  colors.coordinate(withNormalizedOffset: CGVector(dx: 0.5, dy: 0.025)).click(
    forDuration: 0.1,
    thenDragTo: mainWindow.coordinate(withNormalizedOffset: CGVector(dx: 0.25, dy: 0.2)))
  colors.buttons["Color Sliders"].click()
  colors.popUpButtons.firstMatch.click()
  app.menuItems["RGB Sliders"].click()
  colors.menuButtons["color profile"].click()
  XCTContext.runActivity(named: "Native color profile menu") { activity in
    let tree = XCTAttachment(string: app.debugDescription)
    tree.lifetime = .keepAlways
    activity.add(tree)
    let screenshot = XCTAttachment(screenshot: mainWindow.screenshot())
    screenshot.lifetime = .keepAlways
    activity.add(screenshot)
  }
  let profiles = app.menuItems.matching(NSPredicate(format: "title == %@", "sRGB IEC61966-2.1"))
  XCTAssertEqual(profiles.count, 1, "Require an unambiguous native sRGB profile")
  print("Selected native color profile: \(profiles.element.title)")
  profiles.element.click()
  let hexadecimal = colors.textFields["hex"]
  XCTAssertTrue(hexadecimal.waitForExistence(timeout: 5))
  hexadecimal.click()
  hexadecimal.typeKey("a", modifierFlags: .command)
  hexadecimal.typeText(hex)
  hexadecimal.typeKey(.return, modifierFlags: [])
  colors.buttons[XCUIIdentifierCloseWindow].click()
  XCTAssertTrue(colors.waitForNonExistence(timeout: 5))
  if !app.popovers.firstMatch.exists { app.buttons["Style and Selection"].click() }
  XCTAssertTrue(app.popovers.firstMatch.waitForExistence(timeout: 5))
  XCTContext.runActivity(named: "Committed native brush inspector") { activity in
    for (name, element) in [
      ("full-editor-window", mainWindow), ("full-inspector", app.popovers.firstMatch),
    ] {
      let screenshot = XCTAttachment(screenshot: element.screenshot())
      screenshot.name = name
      screenshot.lifetime = .keepAlways
      activity.add(screenshot)
    }
  }
  if app.popovers.firstMatch.exists {
    app.buttons["Style and Selection"].click()
  }
  XCTAssertTrue(app.popovers.firstMatch.waitForNonExistence(timeout: 5))
}

@MainActor
func saveDocument(
  in app: XCUIApplication, to url: URL, saveAs: Bool,
  waitForWrite: Bool = true
) throws {
  if saveAs {
    app.menuBars.menuBarItems["File"].click()
    app.menuItems["Save As…"].click()
  } else {
    app.typeKey("s", modifierFlags: .command)
  }
  try completeNativeSave(in: app, to: url, waitForWrite: waitForWrite)
}

@MainActor func completeNativeSave(
  in app: XCUIApplication, to url: URL,
  waitForWrite: Bool = true
) throws {
  let panel = nativeFilePanel(in: app)
  let name = panel.textFields["saveAsNameTextField"]
  XCTAssertTrue(name.waitForExistence(timeout: 5))
  name.click()
  name.typeKey("a", modifierFlags: .command)
  name.typeText(url.lastPathComponent)
  try navigateFilePanel(app, to: url.deletingLastPathComponent())
  XCTAssertTrue(panel.buttons["Save"].isEnabled)
  panel.buttons["Save"].click()
  XCTAssertTrue(panel.waitForNonExistence(timeout: 5))
  guard waitForWrite else { return }
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
  try completeNativeOpen(in: app, at: url)
}

@MainActor func completeNativeOpen(in app: XCUIApplication, at url: URL) throws {
  let panel = nativeFilePanel(in: app)
  try navigateFilePanel(app, to: url)
  panel.buttons["Open"].click()
  XCTAssertTrue(panel.waitForNonExistence(timeout: 5))
}

@MainActor func nativeFilePanel(in app: XCUIApplication) -> XCUIElement {
  let dialog = app.dialogs.firstMatch
  let sheet = app.sheets.firstMatch
  let appeared = XCTNSPredicateExpectation(
    predicate: NSPredicate { _, _ in dialog.exists || sheet.exists }, object: nil)
  XCTAssertEqual(XCTWaiter.wait(for: [appeared], timeout: 5), .completed)
  return dialog.exists ? dialog : sheet
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
