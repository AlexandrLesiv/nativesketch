import CryptoKit
import Foundation

@MainActor final class SharedDrawingCatalog {
  let directory: URL
  init(directory: URL) { self.directory = directory }
  func list() throws -> [SourceDescriptor] {
    guard FileManager.default.fileExists(atPath: directory.path) else { return [] }
    let urls = try FileManager.default.contentsOfDirectory(
      at: directory, includingPropertiesForKeys: nil
    ).filter { $0.pathExtension == "json" }.prefix(128)
    return urls.compactMap { try? load($0).descriptor }.sorted {
      $0.title.localizedStandardCompare($1.title) == .orderedAscending
    }
  }
  func share(_ data: Data, title: String, documentID: String) throws -> SourceCopy {
    let hash = SourceDescriptor.hash(data)
    let url = directory.appendingPathComponent(
      SourceDescriptor.hash(Data(documentID.utf8)) + ".json")
    let previous = try? load(url)
    if let previous, previous.descriptor.contentSHA256 == hash { return previous }
    let descriptor = SourceDescriptor(
      namespaceID: "phone-local", documentID: documentID, revisionID: UUID().uuidString,
      parentRevisionID: previous?.descriptor.revisionID ?? "", contentSHA256: hash,
      serverGeneration: 0, kind: 0, title: String(title.prefix(512)))
    let copy = SourceCopy(descriptor: descriptor, document: data)
    try copy.validate()
    try FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true)
    if !FileManager.default.fileExists(atPath: url.path), try list().count >= 128 {
      throw SourceFailure.oversized
    }
    try JSONEncoder().encode(copy).write(to: url, options: .atomic)
    return copy
  }
  func fetch(_ requested: SourceDescriptor) throws -> SourceCopy {
    try requested.validate()
    guard requested.namespaceID == "phone-local" else { throw SourceFailure.metadata }
    let copy = try load(
      directory.appendingPathComponent(
        SourceDescriptor.hash(Data(requested.documentID.utf8)) + ".json"))
    guard copy.descriptor.id == requested.id else { throw SourceFailure.rejected }
    return copy
  }
  private func load(_ url: URL) throws -> SourceCopy {
    let size = try url.resourceValues(forKeys: [.fileSizeKey]).fileSize ?? Int.max
    guard size <= 12 * 1024 * 1024 else { throw SourceFailure.oversized }
    let copy = try JSONDecoder().decode(SourceCopy.self, from: Data(contentsOf: url))
    try copy.validate()
    return copy
  }
}
