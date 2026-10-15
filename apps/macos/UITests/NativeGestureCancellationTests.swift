import XCTest

@MainActor final class NativeGestureCancellationTests: XCTestCase {
  func testResizeCancelsHeldStrokeAndPreservesSavedDocument() throws {
    try verifyCancellation(resize: true)
  }

  func testDeactivationCancelsHeldStrokeAndPreservesSavedDocument() throws {
    try verifyCancellation(resize: false)
  }

  private func verifyCancellation(resize: Bool) throws {
    continueAfterFailure = false
    let fixture = try NativeSessionFixture()
    defer { fixture.record(in: self) }
    try fixture.launch()
    fixture.draw(count: 1)
    let file = fixture.document("gesture-\(UUID().uuidString)")
    // Reuse the native Save route already exercised by the maintained UI suite.
    // There is no independent CG/AX file-dialog keyboard implementation here.
    try saveDocument(in: fixture.app, to: file, saveAs: false)
    fixture.diskStrokes(1, at: file)
    waitForSaved(fixture)
    let original = try Data(contentsOf: file)
    let native = try NativeGestureWindow(fixture: fixture, file: file)
    defer {
      record(native.projection, name: "final-gesture-projection")
      do { try native.cleanup() } catch { XCTFail("Native gesture cleanup failed: \(error)") }
    }
    record(
      "XCUIAutomation native Save/Undo; CGEvent.postToPid held mouse; AX resize; own runner window activation. No physical pen or direct editor commands.",
      name: "gesture-input-provenance")
    try native.begin()
    fixture.wait { (native.interaction ?? 0) > 0 }
    record(native.projection, name: "active-held-stroke-before-cancellation")
    if resize {
      let before = try native.currentSize()
      let requested = try native.resize()
      fixture.wait { (try? native.currentSize()) == requested }
      XCTAssertNotEqual(before, try native.currentSize())
      record("before=\(before), after=\(try native.currentSize())", name: "actual-window-resize")
    } else {
      native.deactivate()
      fixture.wait { !native.subject.isActive }
      record("subject.isActive=false", name: "actual-subject-deactivation")
    }
    fixture.wait { native.interaction == 0 }
    record(native.projection, name: "cancelled-before-mouse-up")
    try native.pointer.release()
    native.hideFocusWindow()
    fixture.app.activate()
    fixture.wait { native.subject.isActive }
    fixture.strokes(1)
    try saveAndCompare(fixture, file: file, original: original)
    fixture.draw(count: 2, y: 0.65)
    fixture.app.typeKey("s", modifierFlags: .command)
    fixture.diskStrokes(2, at: file)
    waitForSaved(fixture)
    XCTAssertNotEqual(try Data(contentsOf: file), original)
    fixture.app.typeKey("z", modifierFlags: .command)
    fixture.strokes(1)
    try saveAndCompare(fixture, file: file, original: original)
    // A cancelled draft must not leave a hidden no-op history entry between strokes.
    fixture.app.typeKey("z", modifierFlags: .command)
    fixture.strokes(0)
    fixture.app.typeKey("s", modifierFlags: .command)
    fixture.diskStrokes(0, at: file)
    waitForSaved(fixture)
    fixture.app.typeKey("z", modifierFlags: [.command, .shift])
    fixture.strokes(1)
    try saveAndCompare(fixture, file: file, original: original)
  }

  private func saveAndCompare(_ fixture: NativeSessionFixture, file: URL, original: Data) throws {
    fixture.app.typeKey("s", modifierFlags: .command)
    waitForSaved(fixture)
    XCTAssertEqual(try Data(contentsOf: file), original)
  }

  private func waitForSaved(_ fixture: NativeSessionFixture) {
    fixture.wait {
      let value = fixture.canvas.value as? String ?? ""
      return value.contains("Storage idle=true,dirty=false")
        && value.contains("Save outstanding=0,preparing=0")
    }
  }

  private func record(_ value: String, name: String) {
    let attachment = XCTAttachment(string: value)
    attachment.name = name
    attachment.lifetime = .keepAlways
    add(attachment)
  }
}
