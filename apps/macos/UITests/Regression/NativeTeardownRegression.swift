import XCTest

/// Intentionally failing Apple XCTest cases. The background runner validates their trace
/// and expected failures; this bundle never launches or controls an application.
@MainActor final class NativeTeardownRegression: XCTestCase {
  func testAssertionAbortRunsCleanup() throws {
    continueAfterFailure = false
    let teardown = NativeTestTeardown(test: self)
    teardown.add { try Self.trace("assertion:cleanup") }
    try Self.trace("assertion:body")
    XCTFail("Expected negative regression: assertion abort")
    try Self.trace("assertion:unexpected-after-failure")
  }

  func testThrowingCleanupPreservesLIFO() throws {
    continueAfterFailure = false
    let teardown = NativeTestTeardown(test: self)
    teardown.add { try Self.trace("throwing:outer") }
    teardown.add {
      try Self.trace("throwing:inner")
      throw NSError(domain: "ExpectedTeardownRegression", code: 1)
    }
    try Self.trace("throwing:body")
  }

  private static func trace(_ event: String) throws {
    guard let path = ProcessInfo.processInfo.environment["NATIVE_SKETCH_TEARDOWN_TRACE"],
      path.hasPrefix("/")
    else { throw NSError(domain: "MissingRegressionTrace", code: 1) }
    let handle = try FileHandle(forWritingTo: URL(fileURLWithPath: path))
    try handle.seekToEnd()
    let data = try JSONSerialization.data(withJSONObject: ["event": event])
    try handle.write(contentsOf: data + Data([10]))
    try handle.close()
  }
}
