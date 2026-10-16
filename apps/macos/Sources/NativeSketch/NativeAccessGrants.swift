import AppKit
import CSketchSession

@MainActor final class NativeAccessGrants {
  private struct Grant {
    let url: URL
    let scoped: Bool
  }
  private var grants: [UInt64: Grant] = [:]
  private var next: UInt64 = 0

  func retain(_ url: URL) throws -> UInt64 {
    guard next < UInt64.max, grants.count < 256 else {
      throw NativeBridgeError(operation: "grant-capacity", status: SK_V1_CAPACITY.rawValue)
    }
    next += 1
    grants[next] = Grant(url: url, scoped: url.startAccessingSecurityScopedResource())
    return next
  }

  func url(_ id: UInt64) -> URL? { grants[id]?.url }

  func release(_ id: UInt64) {
    guard let grant = grants.removeValue(forKey: id) else {
      NativeDiagnostics.failure("grant-release", status: SK_V1_NOT_FOUND.rawValue)
      return
    }
    if grant.scoped { grant.url.stopAccessingSecurityScopedResource() }
  }
}
