import XCTest

@MainActor final class NativeRecoveryTests: XCTestCase {
  private var fixture: NativeSessionFixture!
  override func setUp() async throws {
    continueAfterFailure = false
    fixture = try NativeSessionFixture()
  }
  override func tearDown() async throws { fixture?.record(in: self) }

  private func checkpoints() -> [URL] {
    let root = fixture.runtimeRoot.appendingPathComponent("Recovery")
    let enumerator = FileManager.default.enumerator(at: root, includingPropertiesForKeys: nil)
    return (enumerator?.allObjects as? [URL] ?? []).filter { $0.pathExtension == "nskr" }
  }
  private func showRecovery() -> XCUIElement {
    let f = fixture!
    f.app.menuBars.menuBarItems["File"].click()
    f.app.menuItems["Recover Drawings…"].click()
    let table = f.app.tables["sketch.recovery.list"]
    XCTAssertTrue(table.waitForExistence(timeout: 5))
    f.wait { !f.app.staticTexts["sketch.recovery.status"].label.contains("Loading") }
    return table
  }
  func testLaterPreservesRecoveryAndOpensDirtyUnboundCopy() throws {
    let f = fixture!
    try f.launch()
    f.draw(count: 1)
    f.wait { self.checkpoints().count == 1 }
    let checkpoint = try XCTUnwrap(checkpoints().first)
    let bytes = try Data(contentsOf: checkpoint)
    f.app.terminate()  // Abrupt automation termination, not a claim of crash-hardware testing.
    try f.launch()
    XCTAssertEqual(showRecovery().tableRows.count, 1)
    f.app.buttons["sketch.recovery.later"].click()
    XCTAssertEqual(try Data(contentsOf: checkpoint), bytes)
    let table = showRecovery()
    table.tableRows.firstMatch.click()
    f.app.buttons["sketch.recovery.open"].click()
    f.strokes(1)
    XCTAssertTrue((f.canvas.value as? String ?? "").contains("dirty=true"))
    XCTAssertTrue(f.app.windows.firstMatch.title.contains("Untitled"))
    let destination = f.document("recovered-copy")
    try saveDocument(in: f.app, to: destination, saveAs: false)
    f.diskStrokes(1, at: destination)
    XCTAssertTrue(FileManager.default.fileExists(atPath: checkpoint.path))
  }
  func testCorruptNewestGenerationFallsBackToEarlierValidCopy() throws {
    let f = fixture!
    try f.launch()
    f.draw(count: 1)
    f.wait { self.checkpoints().count == 1 }
    let first = try XCTUnwrap(checkpoints().first)
    let good = try Data(contentsOf: first)
    f.draw(count: 2, y: 0.65)
    f.wait { self.checkpoints().count == 2 }
    f.app.terminate()
    let newest = try XCTUnwrap(checkpoints().first { $0 != first })
    try Data("corrupt disposable checkpoint".utf8).write(to: newest)
    try f.launch()
    let table = showRecovery()
    XCTAssertEqual(table.tableRows.count, 1)
    XCTAssertTrue(
      table.staticTexts.containing(NSPredicate(format: "value CONTAINS 'earlier valid copy'"))
        .firstMatch.exists)
    table.tableRows.firstMatch.click()
    f.app.buttons["sketch.recovery.open"].click()
    f.strokes(1)
    XCTAssertEqual(try Data(contentsOf: first), good)
    let destination = f.document("fallback-copy")
    try saveDocument(in: f.app, to: destination, saveAs: false)
    f.diskStrokes(1, at: destination)
  }
  func testDiscardConfirmationDefaultsToCancelAndRemovesOnlySelectedCopy() throws {
    let f = fixture!
    try f.launch()
    f.draw(count: 1)
    f.wait { self.checkpoints().count == 1 }
    f.app.typeKey("n", modifierFlags: .command)
    nativeFilePanel(in: f.app).buttons["Don’t Save"].click()
    f.strokes(0)
    f.draw(count: 1)
    f.draw(count: 2, y: 0.65)
    f.wait { self.checkpoints().count >= 2 }
    f.app.terminate()
    try f.launch()
    let table = showRecovery()
    XCTAssertEqual(table.tableRows.count, 2)
    let selected = table.staticTexts.containing(NSPredicate(format: "value CONTAINS 'revision 2'"))
      .firstMatch
    XCTAssertTrue(selected.exists)
    selected.click()
    f.app.buttons["sketch.recovery.discard"].click()
    f.app.typeKey(.return, modifierFlags: [])
    XCTAssertEqual(
      table.tableRows.count, 2, "Return must choose the non-destructive Cancel default")
    f.app.buttons["sketch.recovery.discard"].click()
    f.app.buttons["Discard"].click()
    f.wait { table.tableRows.count == 1 }
    XCTAssertTrue(
      table.staticTexts.containing(NSPredicate(format: "value CONTAINS 'revision 1'"))
        .firstMatch.exists)
    f.app.buttons["sketch.recovery.later"].click()
  }
}
