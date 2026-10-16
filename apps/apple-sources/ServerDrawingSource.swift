import CryptoKit
import Foundation

@MainActor final class ServerDrawingSource: DrawingSource {
  let base: URL?
  init(base: URL?) {
    if let base, base.scheme == "https", base.host != nil, base.user == nil, base.password == nil,
      base.query == nil, base.fragment == nil
    {
      self.base = base
    } else {
      self.base = nil
    }
  }
  func list() async throws -> [SourceDescriptor] {
    guard let base else { throw SourceFailure.unconfigured }
    let data = try await SourceHTTPTransfer.get(
      base.appendingPathComponent("drawings"), limit: 256 * 1024)
    let result = try JSONDecoder().decode([SourceDescriptor].self, from: data)
    guard result.count <= 128 else { throw SourceFailure.oversized }
    for item in result {
      try item.validate()
      guard !item.phoneOnly else { throw SourceFailure.metadata }
    }
    return result
  }
  func fetch(_ descriptor: SourceDescriptor) async throws -> SourceCopy {
    guard !descriptor.phoneOnly else { throw SourceFailure.phoneOnly }
    try descriptor.validate()
    guard let base else { throw SourceFailure.unconfigured }
    // Escape each ID as exactly one path component, including slash and percent.
    let safe = CharacterSet.alphanumerics.union(CharacterSet(charactersIn: "-_"))
    guard let id = descriptor.documentID.addingPercentEncoding(withAllowedCharacters: safe),
      var parts = URLComponents(
        url: base.appendingPathComponent("drawings"), resolvingAgainstBaseURL: false)
    else { throw SourceFailure.metadata }
    parts.percentEncodedPath += "/" + id
    parts.queryItems = [
      URLQueryItem(name: "namespace", value: descriptor.namespaceID),
      URLQueryItem(name: "revision", value: descriptor.revisionID),
    ]
    guard let url = parts.url else { throw SourceFailure.metadata }
    let data = try await SourceHTTPTransfer.get(url, limit: 8 * 1024 * 1024)
    let copy = SourceCopy(descriptor: descriptor, document: data)
    try copy.validate()
    return copy
  }
}
