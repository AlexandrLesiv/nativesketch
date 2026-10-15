import AppKit
import XCTest

@MainActor
final class EditorJourneyTests: XCTestCase {
  private var app: XCUIApplication!
  private var document: URL!

  override func setUp() async throws {
    continueAfterFailure = false
    let environment = ProcessInfo.processInfo.environment
    app = XCUIApplication(
      url: URL(fileURLWithPath: try XCTUnwrap(environment["NATIVE_SKETCH_APP"])))
    app.launchArguments = ["-AppleLanguages", "(en)", "-AppleLocale", "en_US"]
    app.launchArguments += [
      "--runtime-root",
      FileManager.default.temporaryDirectory.appendingPathComponent(
        "native-ui-runtime-\(UUID().uuidString)"
      ).path,
    ]
    document = URL(fileURLWithPath: try XCTUnwrap(environment["NATIVE_SKETCH_DOCUMENT"]))
  }

  override func tearDown() async throws {
    guard let app else { return }
    keep(app.screenshot(), named: "final-native-window")
    let tree = XCTAttachment(string: app.debugDescription)
    tree.name = "accessibility-tree"
    tree.lifetime = .keepAlways
    add(tree)
    app.terminate()
  }

  func testDrawSaveReopen() throws {
    app.launch()
    let canvas = app.groups["sketch.canvas"]
    XCTAssertTrue(canvas.waitForExistence(timeout: 15))
    let firstPID = try processIdentifier()
    try chooseBrush(in: app, width: "12", hex: "FF0000")
    app.typeKey("n", modifierFlags: .command)
    let canvasFrame = canvas.frame
    let blank = canvas.screenshot()
    keep(blank, named: "before-native-mouse-drag")
    XCTAssertEqual(try redInkPixels(blank), [0, 0, 0], "Expected blank trajectory before drawing")
    let start = canvas.coordinate(withNormalizedOffset: CGVector(dx: 0.3, dy: 0.4))
    let end = canvas.coordinate(withNormalizedOffset: CGVector(dx: 0.7, dy: 0.6))
    // AppKit receives mouse events; XCUI press belongs to its separate touch API.
    start.click(forDuration: 0.1, thenDragTo: end)
    waitForStroke(in: canvas)
    XCTAssertTrue(StrokeExpectation.containsInkInEveryPatch(try redInkPixels(canvas.screenshot())))
    keep(canvas.screenshot(), named: "drawn-red-ink")
    try saveDocument(in: app, to: document, saveAs: false)
    let bytes = try Data(contentsOf: document)
    XCTAssertEqual(canvas.frame, canvasFrame, "Unexpected viewport remapping during the journey")
    try verifyMouseStroke(bytes, canvasSize: canvasFrame.size)
    app.typeKey("q", modifierFlags: .command)
    XCTAssertTrue(app.wait(for: .notRunning, timeout: 10), "Quit must finish normally")
    app.launch()
    XCTAssertNotEqual(try processIdentifier(), firstPID)
    XCTAssertTrue(canvas.waitForExistence(timeout: 15))
    XCTAssertEqual(try redInkPixels(canvas.screenshot()), [0, 0, 0])
    XCTAssertEqual(canvas.frame.size, canvasFrame.size, "Relaunch changed the canvas size")
    try openDocument(in: app, at: document)
    waitForStroke(in: canvas)
    XCTAssertTrue(StrokeExpectation.containsInkInEveryPatch(try redInkPixels(canvas.screenshot())))
    keep(canvas.screenshot(), named: "reopened-red-ink")
    let second = document.deletingLastPathComponent().appendingPathComponent("reopened.nsketch")
    try saveDocument(in: app, to: second, saveAs: true)
    XCTAssertEqual(try Data(contentsOf: second), bytes, "Reopened geometry/style must roundtrip")
  }

  private func processIdentifier() throws -> pid_t {
    let path = try XCTUnwrap(ProcessInfo.processInfo.environment["NATIVE_SKETCH_APP"])
    let expected = URL(fileURLWithPath: path).resolvingSymlinksInPath()
    let process = NSWorkspace.shared.runningApplications.first {
      $0.bundleURL?.resolvingSymlinksInPath() == expected
    }
    return try XCTUnwrap(process).processIdentifier
  }

  private func waitForStroke(in canvas: XCUIElement) {
    let committed = expectation(
      for: NSPredicate(format: "value CONTAINS '1 strokes'"),
      evaluatedWith: canvas)
    wait(for: [committed], timeout: 10)
  }

  private func keep(_ screenshot: XCUIScreenshot, named name: String) {
    let attachment = XCTAttachment(screenshot: screenshot)
    attachment.name = name
    attachment.lifetime = .keepAlways
    add(attachment)
  }
}
