import CSketchSession
import Foundation
import OSLog

/// Host boundary diagnostics contain identifiers/status only, never document content or paths.
enum NativeDiagnostics {
  static let logger = Logger(subsystem: "dev.nativesketch", category: "native-boundary")
  static func failure(_ operation: String, status: UInt32, session: UInt64 = 0) {
    logger.error("operation=\(operation, privacy: .public) status=\(status) session=\(session)")
  }
}

struct NativeBridgeError: Error {
  let operation: String
  let status: UInt32
}

func withNativeText<T>(_ text: String, _ body: (SketchTextV1) throws -> T) rethrows -> T {
  try Array(text.utf8).withUnsafeBufferPointer {
    try body(SketchTextV1(bytes: $0.baseAddress, length: UInt32($0.count)))
  }
}
