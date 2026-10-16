import AppKit
import XCTest

/// SKIA-VISUAL-001 / SKIA-COMMIT-001 / SKIA-INVALIDATE-001 / SKIA-SURFACE-001.
/// Native mouse input and screenshot pixels, not direct editor mutation.
@MainActor final class NativeRendererTests: XCTestCase {
  func testVisibleInkUndoRedoAndMinimizeRestore() throws {
    continueAfterFailure = false
    let fixture = try NativeSessionFixture()
    fixture.registerTeardown(in: self)
    try fixture.launch()
    fixture.wait { self.presented(fixture.canvas) > 0 }
    try chooseBrush(in: fixture.app, width: "12", hex: "FF0000")
    let start = fixture.canvas.coordinate(withNormalizedOffset: CGVector(dx: 0.3, dy: 0.4))
    let end = fixture.canvas.coordinate(withNormalizedOffset: CGVector(dx: 0.7, dy: 0.6))
    let before = presented(fixture.canvas)
    start.click(forDuration: 0.1, thenDragTo: end)
    fixture.strokes(1)
    fixture.wait { self.presented(fixture.canvas) > before }
    try assertInk(fixture.canvas, present: true, name: "skia-committed")
    fixture.app.typeKey("z", modifierFlags: .command)
    fixture.strokes(0)
    try assertInk(fixture.canvas, present: false, name: "skia-undo")
    fixture.app.typeKey("z", modifierFlags: [.command, .shift])
    fixture.strokes(1)
    try assertInk(fixture.canvas, present: true, name: "skia-redo")
    let native = try NativeVisibilityWindow()
    fixture.app.windows.firstMatch.buttons["_XCUI:MinimizeWindow"].click()
    fixture.wait { native.minimized == true }
    let hiddenCount = presented(fixture.canvas)
    try native.restore()
    fixture.wait { native.minimized == false }
    fixture.app.activate()
    fixture.wait {
      fixture.app.state == .runningForeground
        && (fixture.canvas.value as? String ?? "").contains("key=true")
        && self.presented(fixture.canvas) > hiddenCount
        && (fixture.canvas.value as? String ?? "").contains("rendererCurrent=true")
    }
    try assertInk(fixture.canvas, present: true, name: "skia-reveal")
  }

  // This injected failure precedes the first empty frame, not a real driver/device loss.
  func testInjectedBackendFailureRecoversWithoutEditing() throws {
    continueAfterFailure = false
    let fixture = try NativeSessionFixture()
    fixture.registerTeardown(in: self)
    try fixture.launch(extraArguments: ["--test-renderer-failure-once"])
    fixture.wait { self.presented(fixture.canvas) > 0 }
    fixture.strokes(0)
    let value = fixture.canvas.value as? String ?? ""
    XCTAssertTrue(value.contains("rendererRecoveries=1,rendererFailures=1"))
    try chooseBrush(in: fixture.app, width: "12", hex: "FF0000")
    fixture.canvas.coordinate(withNormalizedOffset: CGVector(dx: 0.3, dy: 0.4)).click(
      forDuration: 0.1,
      thenDragTo: fixture.canvas.coordinate(withNormalizedOffset: CGVector(dx: 0.7, dy: 0.6)))
    fixture.strokes(1)
    try assertInk(fixture.canvas, present: true, name: "skia-recovered-injected-failure")
  }

  private func presented(_ canvas: XCUIElement) -> UInt64 {
    let value = canvas.value as? String ?? ""
    guard let range = value.range(of: "Renderer=Skia,presented=") else { return 0 }
    return UInt64(value[range.upperBound...].prefix(while: { $0.isNumber })) ?? 0
  }

  func testInjectedFailureAfterCommitPreservesInkAndHistory() throws {
    continueAfterFailure = false
    let fixture = try NativeSessionFixture()
    fixture.registerTeardown(in: self)
    try fixture.launch(extraArguments: ["--test-renderer-failure-after-stroke"])
    fixture.wait { self.presented(fixture.canvas) > 0 }
    try chooseBrush(in: fixture.app, width: "12", hex: "FF0000")
    fixture.canvas.coordinate(withNormalizedOffset: CGVector(dx: 0.3, dy: 0.4)).click(
      forDuration: 0.1,
      thenDragTo: fixture.canvas.coordinate(withNormalizedOffset: CGVector(dx: 0.7, dy: 0.6)))
    fixture.strokes(1)
    fixture.wait {
      let value = fixture.canvas.value as? String ?? ""
      return value.contains("rendererRecoveries=1,rendererFailures=1")
        && value.contains("rendererCurrent=true")
    }
    try assertInk(fixture.canvas, present: true, name: "skia-recovered-committed-ink")
    fixture.app.typeKey("z", modifierFlags: .command)
    fixture.strokes(0)
    try assertInk(fixture.canvas, present: false, name: "skia-recovered-undo")
    fixture.app.typeKey("z", modifierFlags: [.command, .shift])
    fixture.strokes(1)
    try assertInk(fixture.canvas, present: true, name: "skia-recovered-redo")
  }

  private func assertInk(_ canvas: XCUIElement, present: Bool, name: String) throws {
    let predicate = XCTNSPredicateExpectation(
      predicate: NSPredicate { _, _ in
        guard let pixels = try? redInkPixels(canvas.screenshot()) else { return false }
        return present ? StrokeExpectation.containsInkInEveryPatch(pixels) : pixels == [0, 0, 0]
      }, object: nil)
    XCTAssertEqual(XCTWaiter.wait(for: [predicate], timeout: 10), .completed)
    let attachment = XCTAttachment(screenshot: canvas.screenshot())
    attachment.name = name
    attachment.lifetime = .keepAlways
    add(attachment)
  }
}
