import XCTest

@MainActor final class NativeSheetFocusTests: XCTestCase {
  func testRecoveryLaterRestoresOriginalWidthSelection() throws {
    continueAfterFailure = false
    let f = try NativeSessionFixture()
    f.registerTeardown(in: self)
    try f.launch()
    let width = selectFinalWidthDigit(in: f.app)
    f.app.menuBars.menuBarItems["File"].click()
    f.app.menuItems["Recover Drawings…"].click()
    let later = f.app.buttons["Later"]
    XCTAssertTrue(later.waitForExistence(timeout: 5))
    later.click()
    assertRestoredSelection(width, in: f)
  }

  func testFailedSaveCancellationRestoresOriginalWidthSelection() throws {
    try verifyFailureCancellation(useSuccessor: false)
  }

  func testFailedSaveAsSuccessorCancellationRestoresOriginalWidthSelection() throws {
    try verifyFailureCancellation(useSuccessor: true)
  }

  func testRecoveredCopyUnsavedCancellationRestoresOriginalWidthSelection() throws {
    continueAfterFailure = false
    let f = try NativeSessionFixture()
    f.registerTeardown(in: self)
    try f.launch()
    f.draw(count: 1)
    let recovery = f.runtimeRoot.appendingPathComponent("Recovery")
    f.wait {
      let entries = FileManager.default.enumerator(at: recovery, includingPropertiesForKeys: nil)
      return (entries?.allObjects as? [URL] ?? []).contains { $0.pathExtension == "nskr" }
    }
    let width = selectFinalWidthDigit(in: f.app)
    f.app.menuBars.menuBarItems["File"].click()
    f.app.menuItems["Recover Drawings…"].click()
    let table = f.app.tables["sketch.recovery.list"]
    XCTAssertTrue(table.waitForExistence(timeout: 5))
    f.wait { table.tableRows.count == 1 }
    table.tableRows.firstMatch.click()
    f.app.buttons["sketch.recovery.open"].click()
    let prompt = nativeFilePanel(in: f.app)
    XCTAssertTrue(prompt.buttons["Don’t Save"].waitForExistence(timeout: 5))
    prompt.buttons["Cancel"].click()
    assertRestoredSelection(width, in: f)
    f.strokes(1)
  }

  private func verifyFailureCancellation(useSuccessor: Bool) throws {
    continueAfterFailure = false
    let f = try NativeSessionFixture()
    f.registerTeardown(in: self)
    try f.launch()
    f.draw(count: 1)
    let destination = f.document("focus-last-good")
    try saveDocument(in: f.app, to: destination, saveAs: false)
    f.diskStrokes(1, at: destination)
    f.wait { (f.canvas.value as? String ?? "").contains("Storage idle=true,dirty=false") }
    let bytes = try Data(contentsOf: destination)
    f.app.terminate()
    try f.launch(stage: 5, failure: true)
    try openDocument(in: f.app, at: destination)
    f.strokes(1)
    let width = selectFinalWidthDigit(in: f.app)
    f.app.typeKey("s", modifierFlags: .command)
    try f.reached()
    let cancel = f.app.buttons["sketch.failure.cancel"]
    XCTAssertTrue(cancel.waitForExistence(timeout: 5))
    if useSuccessor {
      f.app.buttons["sketch.failure.save-as"].click()
      let chooser = nativeFilePanel(in: f.app)
      XCTAssertTrue(chooser.textFields["saveAsNameTextField"].waitForExistence(timeout: 5))
      chooser.buttons["Cancel"].click()
    } else {
      cancel.click()
    }
    assertRestoredSelection(width, in: f)
    XCTAssertEqual(try Data(contentsOf: destination), bytes)
  }

  private func selectFinalWidthDigit(in app: XCUIApplication) -> XCUIElement {
    app.buttons["Style and Selection"].click()
    let width = app.textFields["sketch.brush.width"]
    XCTAssertTrue(width.waitForExistence(timeout: 5))
    width.click()
    width.typeKey("a", modifierFlags: .command)
    width.typeText("12")
    width.typeKey(.leftArrow, modifierFlags: .shift)
    return width
  }

  private func assertRestoredSelection(_ width: XCUIElement, in f: NativeSessionFixture) {
    f.wait { (f.canvas.value as? String ?? "").contains("field=true") }
    XCTAssertEqual(width.value as? String, "12")
    f.app.typeKey("8", modifierFlags: [])
    XCTAssertEqual(width.value as? String, "18")
  }
}
