import XCTest

@MainActor final class NativeRecoveryDecisionTests: XCTestCase {
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
  private func integer(_ bytes: Data, at offset: Int) -> UInt64 {
    bytes[offset..<(offset + 8)].enumerated().reduce(UInt64(0)) {
      $0 | UInt64($1.element) << (8 * $1.offset)
    }
  }
  private func waitForNewerCopies(except retained: URL) {
    let originalIdentity = retained.deletingLastPathComponent()
    fixture.wait {
      var latest: [URL: UInt64] = [:]
      for file in self.checkpoints() {
        guard let bytes = try? Data(contentsOf: file), bytes.count >= 64 else { return false }
        let identity = file.deletingLastPathComponent()
        latest[identity] = max(latest[identity] ?? 0, self.integer(bytes, at: 32))
      }
      let others = latest.filter { $0.key != originalIdentity }
      return !others.isEmpty && others.allSatisfy { $0.value >= 2 }
        && (self.fixture.canvas.value as? String ?? "").contains("Storage idle=true")
    }
  }
  private func requestEarlierCopy() {
    let f = fixture!
    f.app.menuBars.menuBarItems["File"].click()
    f.app.menuItems["Recover Drawings…"].click()
    let table = f.app.tables["sketch.recovery.list"]
    XCTAssertTrue(table.waitForExistence(timeout: 5))
    f.wait {
      (f.app.staticTexts["sketch.recovery.status"].value as? String ?? "")
        .contains("recovery copies.")
    }
    let earlier = table.staticTexts.containing(NSPredicate(format: "value CONTAINS 'revision 1'"))
    XCTAssertEqual(earlier.count, 1, "Only the retained original may remain at revision one")
    earlier.element.click()
    f.app.buttons["sketch.recovery.open"].click()
  }
  func testRecoveredCopyOverDirtyDrawingHonorsCancelDiscardAndSave() throws {
    let f = fixture!
    try f.launch()
    f.draw(count: 1, y: 0.2)
    f.wait { self.checkpoints().count == 1 }
    let retained = try XCTUnwrap(checkpoints().first)
    let retainedBytes = try Data(contentsOf: retained)
    XCTAssertGreaterThanOrEqual(retainedBytes.count, 88)
    let payloadStart = 64 + Int(integer(retainedBytes, at: 48))
    let payloadLength = Int(integer(retainedBytes, at: 56))
    XCTAssertEqual(retainedBytes.count, payloadStart + payloadLength + 8)
    let expectedDocument = retainedBytes[payloadStart..<(payloadStart + payloadLength)]
    f.app.typeKey("n", modifierFlags: .command)
    nativeFilePanel(in: f.app).buttons["Don’t Save"].click()
    f.strokes(0)
    f.draw(count: 1)
    f.draw(count: 2, y: 0.6)
    waitForNewerCopies(except: retained)

    requestEarlierCopy()
    nativeFilePanel(in: f.app).buttons["Cancel"].click()
    f.strokes(2)
    XCTAssertTrue((f.canvas.value as? String ?? "").contains("dirty=true"))
    requestEarlierCopy()
    nativeFilePanel(in: f.app).buttons["Don’t Save"].click()
    f.strokes(1)
    XCTAssertTrue((f.canvas.value as? String ?? "").contains("dirty=true"))
    XCTAssertTrue(f.app.windows.firstMatch.title.contains("Untitled"))
    XCTAssertEqual(try Data(contentsOf: retained), retainedBytes)

    f.draw(count: 2, y: 0.6)
    waitForNewerCopies(except: retained)
    requestEarlierCopy()
    nativeFilePanel(in: f.app).buttons["Save"].click()
    let destination = f.document("before-recovery")
    try completeNativeSave(in: f.app, to: destination)
    f.diskStrokes(2, at: destination)
    f.strokes(1)
    XCTAssertTrue((f.canvas.value as? String ?? "").contains("dirty=true"))
    XCTAssertTrue(f.app.windows.firstMatch.title.contains("Untitled"))
    XCTAssertEqual(try Data(contentsOf: retained), retainedBytes)
    let recovered = f.document("recovered-after-save")
    try saveDocument(in: f.app, to: recovered, saveAs: false)
    f.diskStrokes(1, at: recovered)
    XCTAssertEqual(try Data(contentsOf: recovered), expectedDocument)
    f.diskStrokes(2, at: destination)
  }
}
