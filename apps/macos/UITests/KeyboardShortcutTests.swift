import AppKit
import XCTest

@MainActor final class KeyboardShortcutTests: XCTestCase {
  private var app: XCUIApplication!
  private var document: URL!

  override func setUp() async throws {
    continueAfterFailure = false
    let environment = ProcessInfo.processInfo.environment
    app = XCUIApplication(
      url: URL(fileURLWithPath: try XCTUnwrap(environment["NATIVE_SKETCH_APP"])))
    app.launchArguments = [
      "-AppleLanguages", "(en)", "-AppleLocale", "en_US", "--native-ui-observation",
    ]
    let root = URL(fileURLWithPath: try XCTUnwrap(environment["NATIVE_SKETCH_DOCUMENT"]))
      .deletingLastPathComponent()
    document = root.appendingPathComponent("shortcut-\(UUID().uuidString).nsketch")
    app.launchArguments += [
      "--runtime-root", root.appendingPathComponent("runtime-\(UUID().uuidString)").path,
    ]
    app.launch()
    XCTAssertTrue(canvas.waitForExistence(timeout: 15))
    assertCanvasKeyboardFocus()
  }

  override func tearDown() async throws {
    guard let app else { return }
    let trace = XCTAttachment(string: canvas.value as? String ?? "No canvas value")
    trace.name = "native-focus-observation"
    trace.lifetime = .keepAlways
    add(trace)
    let tree = XCTAttachment(string: app.debugDescription)
    tree.name = "keyboard-final-accessibility"
    tree.lifetime = .keepAlways
    add(tree)
    let screenshot = XCTAttachment(screenshot: app.screenshot())
    screenshot.lifetime = .keepAlways
    add(screenshot)
    app.terminate()
  }

  private var canvas: XCUIElement { app.groups["sketch.canvas"] }

  func testSaveAsKeyboardBeforeAndAfterBinding() throws {
    // No Save warmup: observe native activation, key window and first responder.
    try openAndCancelSaveAs(expectedName: "Untitled.nsketch")
    waitForStrokeCount(0)
    drawStroke()
    try saveDocument(in: app, to: document, saveAs: false)
    let saved = try Data(contentsOf: document)
    try openAndCancelSaveAs(expectedName: document.lastPathComponent)
    XCTAssertEqual(try Data(contentsOf: document), saved)
    waitForStrokeCount(1)
    XCTAssertTrue(app.windows.firstMatch.title.contains(document.lastPathComponent))
    // Require a real edit so disabled/no-op Save cannot satisfy the binding oracle.
    let directory = document.deletingLastPathComponent()
    let filesBefore = try FileManager.default.contentsOfDirectory(atPath: directory.path).sorted()
    app.typeKey("z", modifierFlags: .command)
    waitForStrokeCount(0)
    app.typeKey("s", modifierFlags: .command)
    XCTAssertFalse(app.dialogs.firstMatch.waitForExistence(timeout: 2))
    XCTAssertFalse(app.sheets.firstMatch.exists)
    let written = XCTNSPredicateExpectation(
      predicate: NSPredicate { _, _ in
        guard let bytes = try? Data(contentsOf: self.document), bytes.count >= 16 else {
          return false
        }
        return Array(bytes[8..<16]) == Array(repeating: UInt8(0), count: 8)
      }, object: nil)
    XCTAssertEqual(XCTWaiter.wait(for: [written], timeout: 10), .completed)
    let updated = try Data(contentsOf: document)
    XCTAssertNotEqual(updated, saved)
    XCTAssertEqual(String(data: updated.prefix(8), encoding: .utf8), "NSKETCH1")
    guard updated.count >= 16 else { return XCTFail("Truncated saved document header") }
    XCTAssertEqual(Array(updated[8..<16]), Array(repeating: UInt8(0), count: 8))
    XCTAssertEqual(
      try FileManager.default.contentsOfDirectory(atPath: directory.path).sorted(), filesBefore)
    XCTAssertTrue(app.windows.firstMatch.title.contains(document.lastPathComponent))
  }

  func testOneStrokeUndoRedoKeyboard() throws {
    drawStroke()
    try saveDocument(in: app, to: document, saveAs: false)
    let committed = try Data(contentsOf: document)
    app.typeKey("z", modifierFlags: .command)
    waitForStrokeCount(0)
    let undone = try saveFreshSnapshot()
    guard undone.count >= 16 else {
      XCTFail("Fresh Undo snapshot has a truncated document header")
      return
    }
    XCTAssertEqual(String(data: undone.prefix(8), encoding: .utf8), "NSKETCH1")
    XCTAssertEqual(Array(undone[8..<16]), Array(repeating: UInt8(0), count: 8))
    app.typeKey("z", modifierFlags: [.command, .shift])
    waitForStrokeCount(1)
    XCTAssertEqual(try saveFreshSnapshot(), committed)
    app.typeKey("z", modifierFlags: .command)
    waitForStrokeCount(0)
  }

  func testFieldEditingKeepsCanvasState() throws {
    drawStroke()
    try saveDocument(in: app, to: document, saveAs: false)
    let committed = try Data(contentsOf: document)
    app.menuBars.menuBarItems["Edit"].click()
    app.menuItems["Select Next Stroke"].click()
    XCTAssertTrue((canvas.value as? String ?? "").contains("one selected"))
    app.buttons["Style and Selection"].click()
    let width = app.textFields["sketch.brush.width"]
    XCTAssertTrue(width.waitForExistence(timeout: 5))
    width.click()
    let fieldFocused = expectation(
      for: NSPredicate(format: "value CONTAINS 'field=true'"), evaluatedWith: canvas)
    wait(for: [fieldFocused], timeout: 5)
    width.typeKey("a", modifierFlags: .command)
    width.typeText("12")
    XCTAssertEqual(width.value as? String, "12")
    width.typeKey("z", modifierFlags: .command)
    XCTAssertNotEqual(width.value as? String, "12", "Native field Undo must change its text")
    width.typeKey("z", modifierFlags: [.command, .shift])
    XCTAssertEqual(width.value as? String, "12", "Native field Redo must restore its text")
    waitForStrokeCount(1)
    width.typeKey("a", modifierFlags: .command)
    width.typeText("p")
    XCTAssertEqual(app.radioButtons["Select"].value as? NSNumber, NSNumber(value: true))
    width.typeText("v ")
    width.typeKey(.delete, modifierFlags: [])
    waitForStrokeCount(1)
    XCTAssertEqual(app.radioButtons["Pen"].value as? NSNumber, NSNumber(value: false))
    XCTAssertEqual(app.radioButtons["Select"].value as? NSNumber, NSNumber(value: true))
    XCTAssertTrue((canvas.value as? String ?? "").contains("one selected"))
    width.typeKey("a", modifierFlags: .command)
    width.typeText("6")
    width.typeKey(.return, modifierFlags: [])
    app.buttons["Style and Selection"].click()
    XCTAssertTrue(app.popovers.firstMatch.waitForNonExistence(timeout: 5))
    XCTAssertEqual(try saveFreshSnapshot(), committed)
  }

  private func saveFreshSnapshot() throws -> Data {
    let destination = document.deletingLastPathComponent()
      .appendingPathComponent("snapshot-\(UUID().uuidString).nsketch")
    XCTAssertFalse(FileManager.default.fileExists(atPath: destination.path))
    try saveDocument(in: app, to: destination, saveAs: true)
    return try Data(contentsOf: destination)
  }

  private func openAndCancelSaveAs(expectedName: String) throws {
    app.typeKey("s", modifierFlags: [.command, .shift])
    let panel = nativeFilePanel(in: app)
    let name = panel.textFields["saveAsNameTextField"]
    XCTAssertEqual(try XCTUnwrap(name.value as? String), expectedName)
    let screenshot = XCTAttachment(screenshot: panel.screenshot())
    screenshot.name = "keyboard-save-as-panel"
    screenshot.lifetime = .keepAlways
    add(screenshot)
    panel.buttons["Cancel"].click()
    XCTAssertTrue(panel.waitForNonExistence(timeout: 5))
    assertCanvasKeyboardFocus()
  }

  private func assertCanvasKeyboardFocus() {
    // Observe AppKit state through XCUI. Direct runner AX access is separately unavailable.
    let ready = expectation(
      for: NSPredicate(
        format: "value CONTAINS %@", "Native ready:active=true,key=true,canvas=true"),
      evaluatedWith: canvas)
    wait(for: [ready], timeout: 5)
  }

  private func drawStroke() {
    let start = canvas.coordinate(withNormalizedOffset: CGVector(dx: 0.3, dy: 0.4))
    let end = canvas.coordinate(withNormalizedOffset: CGVector(dx: 0.7, dy: 0.6))
    start.click(forDuration: 0.1, thenDragTo: end)
    waitForStrokeCount(1)
  }

  private func waitForStrokeCount(_ count: Int) {
    let committed = expectation(
      for: NSPredicate(format: "value BEGINSWITH %@", "\(count) strokes,"), evaluatedWith: canvas)
    wait(for: [committed], timeout: 5)
  }
}
