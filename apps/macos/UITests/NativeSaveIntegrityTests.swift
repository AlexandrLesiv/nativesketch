import XCTest

/// Native destination binding and last-good-byte assertions, using one-shot worker faults.
@MainActor final class NativeSaveIntegrityTests: XCTestCase {
  private var fixture: NativeSessionFixture!
  override func setUp() async throws {
    continueAfterFailure = false
    fixture = try NativeSessionFixture()
  }
  override func tearDown() async throws { fixture?.record(in: self) }

  private func boundDocument(failure: Bool) throws -> URL {
    let f = fixture!
    try f.launch()
    f.draw(count: 1)
    let original = f.document("original")
    try saveDocument(in: f.app, to: original, saveAs: false)
    f.diskStrokes(1, at: original)
    f.app.terminate()
    try f.launch(stage: 5, failure: failure)
    try openDocument(in: f.app, at: original)
    f.strokes(1)
    return original
  }

  func testOrdinarySaveDuringPendingSaveAsWritesOldBinding() throws {
    let f = fixture!
    let original = try boundDocument(failure: false)
    f.draw(count: 2, y: 0.6)
    let destination = f.document("pending-save-as")
    try saveDocument(in: f.app, to: destination, saveAs: true, waitForWrite: false)
    try f.reached()
    f.draw(count: 3, y: 0.2)
    f.app.typeKey("s", modifierFlags: .command)
    f.wait { (f.canvas.value as? String ?? "").contains("Save outstanding=2,preparing=0") }
    XCTAssertTrue(f.app.windows.firstMatch.title.contains(original.lastPathComponent))
    XCTAssertFalse(f.app.sheets.firstMatch.exists)
    try f.release()
    f.diskStrokes(2, at: destination)
    f.diskStrokes(3, at: original)
    f.wait { (f.canvas.value as? String ?? "").contains("Storage idle=true") }
    XCTAssertTrue(f.app.windows.firstMatch.title.contains(destination.lastPathComponent))
    XCTAssertTrue((f.canvas.value as? String ?? "").contains("dirty=true"))
    f.app.typeKey("s", modifierFlags: .command)
    f.diskStrokes(3, at: destination)
  }

  func testOrdinarySaveAfterFailedSaveAsKeepsOriginalBinding() throws {
    let f = fixture!
    let original = try boundDocument(failure: true)
    let before = try Data(contentsOf: original)
    f.draw(count: 2, y: 0.6)
    let failed = f.document("failed-save-as")
    try saveDocument(in: f.app, to: failed, saveAs: true, waitForWrite: false)
    try f.reached()
    let cancel = f.app.buttons["sketch.failure.cancel"]
    XCTAssertTrue(cancel.waitForExistence(timeout: 10))
    XCTAssertEqual(try Data(contentsOf: original), before)
    cancel.click()
    f.app.typeKey("s", modifierFlags: .command)
    f.diskStrokes(2, at: original)
    f.wait { (f.canvas.value as? String ?? "").contains("Storage idle=true,dirty=false") }
    XCTAssertTrue(f.app.windows.firstMatch.title.contains(original.lastPathComponent))
    XCTAssertFalse(FileManager.default.fileExists(atPath: failed.path))
    XCTAssertFalse(f.app.sheets.firstMatch.exists)
  }

  func testFailedOverwritePreservesLastGoodBytesAndCanRetry() throws {
    let f = fixture!
    let original = try boundDocument(failure: true)
    let before = try Data(contentsOf: original)
    f.draw(count: 2, y: 0.6)
    f.app.typeKey("s", modifierFlags: .command)
    try f.reached()
    let cancel = f.app.buttons["sketch.failure.cancel"]
    XCTAssertTrue(cancel.waitForExistence(timeout: 10))
    XCTAssertEqual(try Data(contentsOf: original), before)
    XCTAssertTrue((f.canvas.value as? String ?? "").contains("dirty=true"))
    cancel.click()
    f.app.typeKey("s", modifierFlags: .command)
    f.diskStrokes(2, at: original)
    f.wait { (f.canvas.value as? String ?? "").contains("Storage idle=true,dirty=false") }
    XCTAssertFalse(f.app.sheets.firstMatch.exists)
  }
}
