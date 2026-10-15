import XCTest

@MainActor final class NativeReadIsolationTests: XCTestCase {
  private var fixture: NativeSessionFixture!
  override func setUp() async throws {
    continueAfterFailure = false
    fixture = try NativeSessionFixture()
  }
  override func tearDown() async throws { fixture?.record(in: self) }

  private func sourceDocument() throws -> URL {
    let f = fixture!
    try f.launch()
    f.draw(count: 1, y: 0.2)
    f.draw(count: 2, y: 0.6)
    let source = f.document("source-two-strokes")
    try saveDocument(in: f.app, to: source, saveAs: false)
    f.diskStrokes(2, at: source)
    f.app.terminate()
    return source
  }

  func testOversizedOpenRetainsContentAndBinding() throws {
    let f = fixture!
    let source = try sourceDocument()
    let before = try Data(contentsOf: source)
    try f.launch()
    try openDocument(in: f.app, at: source)
    f.strokes(2)
    let oversized = f.document("oversized")
    XCTAssertTrue(FileManager.default.createFile(atPath: oversized.path, contents: Data()))
    let file = try FileHandle(forWritingTo: oversized)
    try file.truncate(atOffset: 64 * 1024 * 1024 + 1)
    try file.close()
    try openDocument(in: f.app, at: oversized)
    let cancel = f.app.buttons["sketch.failure.cancel"]
    XCTAssertTrue(cancel.waitForExistence(timeout: 10))
    cancel.click()
    f.strokes(2)
    XCTAssertTrue(f.app.windows.firstMatch.title.contains(source.lastPathComponent))
    XCTAssertEqual(try Data(contentsOf: source), before)
    f.draw(count: 3)
    f.app.typeKey("s", modifierFlags: .command)
    f.diskStrokes(3, at: source)
    XCTAssertFalse(f.app.sheets.firstMatch.exists)
  }

  func testDelayedValidatedReadCannotReplaceNewerDrawing() throws {
    let f = fixture!
    let source = try sourceDocument()
    let before = try Data(contentsOf: source)
    // Stage 8 holds a real successfully decoded candidate before worker delivery.
    try f.launch(stage: 8)
    try openDocument(in: f.app, at: source)
    try f.reached()
    f.draw(count: 1)
    try f.release()
    f.wait { (f.canvas.value as? String ?? "").contains("Storage idle=true") }
    f.strokes(1)
    XCTAssertTrue(f.app.windows.firstMatch.title.contains("Untitled"))
    XCTAssertFalse(f.app.sheets.firstMatch.exists)
    XCTAssertEqual(try Data(contentsOf: source), before)
    let current = f.document("newer-drawing")
    try saveDocument(in: f.app, to: current, saveAs: false)
    f.diskStrokes(1, at: current)
  }

  func testCancelledSourceResolutionFailureCannotPresentObsoleteError() throws {
    let f = fixture!
    let parent = f.root.appendingPathComponent("removed-source-parent")
    try FileManager.default.createDirectory(at: parent, withIntermediateDirectories: false)
    let corrupt = parent.appendingPathComponent("source.nsketch")
    try Data("invalid drawing".utf8).write(to: corrupt)
    try f.launch(stage: 1)
    try openDocument(in: f.app, at: corrupt)
    try f.reached()
    f.app.menuBars.menuBarItems["File"].click()
    f.app.menuItems["Cancel File Operation"].click()
    // The real held Resolve must fail its parent canonicalization after release.
    // Corrupt contents alone would not fail resolution and would be a false oracle.
    try FileManager.default.removeItem(at: parent)
    f.draw(count: 1)
    try f.release()
    f.wait { (f.canvas.value as? String ?? "").contains("Storage idle=true") }
    XCTAssertFalse(f.app.sheets.firstMatch.exists)
    XCTAssertFalse(f.app.buttons["sketch.failure.cancel"].exists)
    f.strokes(1)
    XCTAssertTrue(f.app.windows.firstMatch.title.contains("Untitled"))
  }

  func testCloseAfterCancellingReadDoesNotResurrectWindow() throws {
    let f = fixture!
    let source = try sourceDocument()
    try f.launch(stage: 8)
    try openDocument(in: f.app, at: source)
    try f.reached()
    f.app.menuBars.menuBarItems["File"].click()
    f.app.menuItems["Cancel File Operation"].click()
    f.app.typeKey("w", modifierFlags: .command)
    try f.release()
    f.wait { f.app.state == .notRunning }
    f.diskStrokes(2, at: source)
  }
}
