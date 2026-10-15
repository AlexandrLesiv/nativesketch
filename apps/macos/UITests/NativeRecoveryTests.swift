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
  private func checkpointMetadata(_ url: URL) -> (revision: UInt64, strokes: UInt64)? {
    guard let bytes = try? Data(contentsOf: url), bytes.count >= 88,
      String(data: bytes.prefix(8), encoding: .utf8) == "NSRECOV1"
    else { return nil }
    func integer(_ offset: Int) -> UInt64 {
      bytes[offset..<(offset + 8)].enumerated().reduce(UInt64(0)) {
        $0 | UInt64($1.element) << (8 * $1.offset)
      }
    }
    let nameLength = integer(48)
    let payloadLength = integer(56)
    guard nameLength <= 256, payloadLength >= 16, payloadLength <= 64 * 1024 * 1024,
      UInt64(bytes.count) == 72 + nameLength + payloadLength
    else { return nil }
    let checksum = bytes.dropLast(8).reduce(UInt64(0xcbf2_9ce4_8422_2325)) {
      ($0 ^ UInt64($1)) &* 0x100_0000_01b3
    }
    guard checksum == integer(bytes.count - 8) else { return nil }
    let payload = 64 + Int(nameLength)
    guard String(data: bytes[payload..<(payload + 8)], encoding: .utf8) == "NSKETCH1"
    else { return nil }
    return (integer(32), integer(payload + 8))
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
    let firstIdentity = try XCTUnwrap(checkpoints().first).deletingLastPathComponent()
    f.app.typeKey("n", modifierFlags: .command)
    nativeFilePanel(in: f.app).buttons["Don’t Save"].click()
    f.strokes(0)
    f.draw(count: 1)
    f.draw(count: 2, y: 0.65)
    f.wait {
      self.checkpoints().contains { url in
        guard url.deletingLastPathComponent() != firstIdentity,
          let metadata = self.checkpointMetadata(url)
        else { return false }
        return metadata.revision == 2 && metadata.strokes == 2
      } && (f.canvas.value as? String ?? "").contains("Storage idle=true")
    }
    f.app.terminate()
    try f.launch()
    let table = showRecovery()
    XCTAssertEqual(table.tableRows.count, 2)
    let selected = table.staticTexts.containing(NSPredicate(format: "value CONTAINS 'revision 2'"))
      .firstMatch
    XCTAssertTrue(selected.exists)
    selected.click()
    f.app.buttons["sketch.recovery.discard"].click()
    let confirmation = f.app.buttons["sketch.recovery.confirm-cancel"]
    XCTAssertTrue(confirmation.waitForExistence(timeout: 5))
    let screenshot = XCTAttachment(screenshot: f.app.screenshot())
    screenshot.name = "native-recovery-discard-confirmation"
    screenshot.lifetime = .keepAlways
    add(screenshot)
    f.app.typeKey(.return, modifierFlags: [])
    XCTAssertTrue(confirmation.waitForNonExistence(timeout: 5))
    f.app.buttons["sketch.recovery.discard"].click()
    XCTAssertTrue(confirmation.waitForExistence(timeout: 5))
    f.app.typeKey(.escape, modifierFlags: [])
    XCTAssertTrue(confirmation.waitForNonExistence(timeout: 5))
    XCTAssertEqual(
      table.tableRows.count, 2, "Return must choose the non-destructive Cancel default")
    f.app.buttons["sketch.recovery.discard"].click()
    f.app.buttons["sketch.recovery.confirm-discard"].click()
    f.wait { table.tableRows.count == 1 }
    XCTAssertTrue(
      table.staticTexts.containing(NSPredicate(format: "value CONTAINS 'revision 1'"))
        .firstMatch.exists)
    f.app.buttons["sketch.recovery.later"].click()
  }
}
