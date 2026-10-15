import CryptoKit
import XCTest

enum NativeTestProvenance {
  /// Verify the loaded bundle, not merely the file the launcher intended to load.
  @MainActor static func verify(for testClass: AnyClass) throws {
    let environment = ProcessInfo.processInfo.environment
    guard let expectedPath = environment["NATIVE_SKETCH_TEST_BUNDLE_PATH"],
      expectedPath.hasPrefix("/"), expectedPath.hasSuffix(".xctest"),
      let expectedHash = environment["NATIVE_SKETCH_TEST_BINARY_SHA256"],
      expectedHash.count == 64, expectedHash.allSatisfy({ "0123456789abcdef".contains($0) })
    else { throw failure("Missing or malformed expected test-bundle provenance") }
    let bundle = Bundle(for: testClass)
    let actualPath = bundle.bundleURL.resolvingSymlinksInPath().path
    guard let executable = bundle.executableURL else {
      throw failure("Loaded test bundle has no executable")
    }
    let bytes = try Data(contentsOf: executable, options: .mappedIfSafe)
    let actualHash = SHA256.hash(data: bytes).map { String(format: "%02x", $0) }.joined()
    let evidence = try JSONSerialization.data(
      withJSONObject: [
        "schemaVersion": 1, "bundlePath": actualPath,
        "executablePath": executable.resolvingSymlinksInPath().path, "binarySha256": actualHash,
      ], options: [.sortedKeys])
    XCTContext.runActivity(named: "Loaded XCTest bundle provenance") { activity in
      let attachment = XCTAttachment(string: String(decoding: evidence, as: UTF8.self))
      attachment.name = "loaded-test-bundle-provenance"
      attachment.lifetime = .keepAlways
      activity.add(attachment)
    }
    guard actualPath == URL(fileURLWithPath: expectedPath).resolvingSymlinksInPath().path else {
      throw failure("Loaded XCTest bundle path differs from the prepared external bundle")
    }
    guard actualHash == expectedHash else {
      throw failure("Loaded XCTest binary differs from its recorded SHA-256")
    }
  }

  private static func failure(_ message: String) -> NSError {
    NSError(domain: "NativeTestProvenance", code: 1, userInfo: [NSLocalizedDescriptionKey: message])
  }
}
