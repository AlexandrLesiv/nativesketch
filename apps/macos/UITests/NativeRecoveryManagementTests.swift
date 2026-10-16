import XCTest

@MainActor final class NativeRecoveryManagementTests: XCTestCase {
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
    f.wait {
      let value = f.app.staticTexts["sketch.recovery.status"].value as? String ?? ""
      return !value.isEmpty && !value.contains("Loading")
    }
    return table
  }

  func testRecoveryCapacityFeedbackRetainsAllExistingCopies() throws {
    let f = fixture!
    try f.launch()
    for count in 1...20 {
      f.draw(count: 1)
      f.wait { self.checkpoints().count == count }
      if count < 20 {
        f.app.typeKey("n", modifierFlags: .command)
        nativeFilePanel(in: f.app).buttons["Don’t Save"].click()
        f.strokes(0)
      }
    }
    let retained = try Dictionary(
      uniqueKeysWithValues: checkpoints().map { ($0, try Data(contentsOf: $0)) })
    XCTAssertEqual(retained.count, 20)
    f.app.typeKey("n", modifierFlags: .command)
    nativeFilePanel(in: f.app).buttons["Don’t Save"].click()
    f.strokes(0)
    f.draw(count: 1)
    let table = showRecovery()
    f.wait {
      (f.app.staticTexts["sketch.recovery.status"].value as? String ?? "")
        .contains("Recovery capacity is full")
    }
    XCTAssertEqual(table.tableRows.count, 20)
    for (url, bytes) in retained { XCTAssertEqual(try Data(contentsOf: url), bytes) }
    f.app.buttons["sketch.recovery.later"].click()
    let document = f.document("drawing-despite-full-recovery")
    try saveDocument(in: f.app, to: document, saveAs: false)
    f.diskStrokes(1, at: document)
    for (url, bytes) in retained { XCTAssertEqual(try Data(contentsOf: url), bytes) }
  }

  func testDiscardCannotRemoveGenerationNewerThanDisplayedCandidate() throws {
    let f = fixture!
    try f.launch()
    f.draw(count: 1)
    f.wait { self.checkpoints().count == 1 }
    let first = try XCTUnwrap(checkpoints().first)
    let firstBytes = try Data(contentsOf: first)
    f.draw(count: 2, y: 0.6)
    f.wait { self.checkpoints().count == 2 }
    f.app.terminate()
    let newer = try XCTUnwrap(checkpoints().first { $0 != first })
    let newerBytes = try Data(contentsOf: newer)
    // Retain two genuine worker-produced generations. Reveal the newer one only
    // after native confirmation has captured the older inventory candidate.
    try FileManager.default.removeItem(at: newer)
    try f.launch()
    let table = showRecovery()
    XCTAssertEqual(table.tableRows.count, 1)
    XCTAssertTrue(
      table.staticTexts.containing(NSPredicate(format: "value CONTAINS 'revision 1'"))
        .firstMatch.exists)
    table.tableRows.firstMatch.click()
    f.app.buttons["sketch.recovery.discard"].click()
    let confirm = f.app.buttons["sketch.recovery.confirm-discard"]
    XCTAssertTrue(confirm.waitForExistence(timeout: 5))
    try newerBytes.write(to: newer, options: .atomic)
    confirm.click()
    f.wait {
      table.staticTexts.containing(NSPredicate(format: "value CONTAINS 'revision 2'"))
        .firstMatch.exists
    }
    XCTAssertEqual(table.tableRows.count, 1)
    XCTAssertEqual(try Data(contentsOf: first), firstBytes)
    XCTAssertEqual(try Data(contentsOf: newer), newerBytes)
    table.tableRows.firstMatch.click()
    f.app.buttons["sketch.recovery.open"].click()
    f.strokes(2)
    let recovered = f.document("newer-preserved")
    try saveDocument(in: f.app, to: recovered, saveAs: false)
    f.diskStrokes(2, at: recovered)
  }
}
