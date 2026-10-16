import CryptoKit
import Foundation

struct SourceDescriptor: Codable, Sendable, Equatable, Identifiable {
  let namespaceID: String
  let documentID: String
  let revisionID: String
  let parentRevisionID: String
  let contentSHA256: String
  let serverGeneration: UInt64
  let kind: UInt32
  let title: String
  var id: String { String(namespaceID.utf8.count) + ":" + namespaceID + documentID }
  var phoneOnly: Bool { kind == 0 && serverGeneration == 0 }
  func validate() throws {
    for value in [namespaceID, documentID, revisionID] {
      guard !value.isEmpty, value.utf8.count <= 256,
        !value.unicodeScalars.contains(where: { CharacterSet.controlCharacters.contains($0) })
      else { throw SourceFailure.metadata }
    }
    guard parentRevisionID.utf8.count <= 256,
      !parentRevisionID.unicodeScalars.contains(where: {
        CharacterSet.controlCharacters.contains($0)
      }), contentSHA256.count == 64,
      contentSHA256.utf8.allSatisfy({ (48...57).contains($0) || (97...102).contains($0) }),
      !title.isEmpty, title.utf8.count <= 2048, kind <= 1, kind == 0 || serverGeneration > 0
    else { throw SourceFailure.metadata }
  }
  static func hash(_ data: Data) -> String {
    SHA256.hash(data: data).map { String(format: "%02x", $0) }.joined()
  }
}
struct SourceCopy: Codable, Sendable {
  let descriptor: SourceDescriptor
  let document: Data
  func validate() throws {
    try descriptor.validate()
    guard document.count <= 8 * 1024 * 1024 else { throw SourceFailure.oversized }
    guard SourceDescriptor.hash(document) == descriptor.contentSHA256 else {
      throw SourceFailure.integrity
    }
  }
}
enum SourceFailure: Error, LocalizedError {
  case metadata, oversized, integrity, unavailable, unconfigured, phoneOnly, rejected, http, timeout
  var errorDescription: String? {
    switch self {
    case .metadata: "Drawing metadata is invalid."
    case .oversized: "The drawing exceeds the Watch size limit."
    case .integrity: "The drawing did not match its content hash."
    case .unavailable: "iPhone is unavailable."
    case .unconfigured: "Server access is not configured. Cached drawings remain available."
    case .phoneOnly: "This copy is available only from iPhone."
    case .rejected: "The update is older, conflicting, or belongs to another drawing."
    case .http: "The server request failed or redirected to another origin."
    case .timeout: "iPhone did not respond in time."
    }
  }
}
@MainActor protocol DrawingSource: AnyObject {
  func list() async throws -> [SourceDescriptor]
  func fetch(_ descriptor: SourceDescriptor) async throws -> SourceCopy
}
