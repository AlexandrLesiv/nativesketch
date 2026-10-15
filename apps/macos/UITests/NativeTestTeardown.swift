import XCTest

/// Main-actor ownership lets XCTest retain cleanup without sending its test case across actors.
@MainActor final class NativeTestTeardown {
  private let test: XCTestCase

  init(test: XCTestCase) { self.test = test }

  /// Framework blocks run LIFO, including after an assertion abort or another block's error.
  func add(_ action: @escaping @MainActor @Sendable () throws -> Void) {
    test.addTeardownBlock { try await action() }
  }

  func attach(_ value: String, name: String) {
    let attachment = XCTAttachment(string: value)
    attachment.name = name
    attachment.lifetime = .keepAlways
    test.add(attachment)
  }

  func attach(_ screenshot: XCUIScreenshot, name: String) {
    let attachment = XCTAttachment(screenshot: screenshot)
    attachment.name = name
    attachment.lifetime = .keepAlways
    test.add(attachment)
  }
}
