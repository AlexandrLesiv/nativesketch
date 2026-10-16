import CSketchFFI
import Foundation

@MainActor final class WatchLibraryStore {
  struct Entry: Codable {
    let copy: SourceCopy
    let receivedAt: Double
    let source: String
  }
  struct Archive: Codable {
    var selected: String?
    var entries: [Entry]
  }
  let url: URL
  private(set) var archive = Archive(selected: nil, entries: [])
  private(set) var document = ReadOnlyDocument()
  var selected: Entry? { archive.entries.first { $0.copy.descriptor.id == archive.selected } }
  init(url: URL) { self.url = url }
  func restore() throws {
    guard FileManager.default.fileExists(atPath: url.path) else { return }
    let data = try boundedRead(url, limit: 48 * 1024 * 1024)
    let loaded = try JSONDecoder().decode(Archive.self, from: data)
    guard loaded.entries.count <= 8 else { throw SourceFailure.oversized }
    var identities = Set<String>()
    for entry in loaded.entries {
      try entry.copy.validate()
      guard entry.receivedAt.isFinite, entry.receivedAt > 0,
        identities.insert(entry.copy.descriptor.id).inserted
      else { throw SourceFailure.metadata }
    }
    guard loaded.selected == nil || identities.contains(loaded.selected!) else {
      throw SourceFailure.metadata
    }
    let candidate = ReadOnlyDocument()
    if let entry = loaded.entries.first(where: { $0.copy.descriptor.id == loaded.selected }) {
      try candidate.importCopy(entry.copy.document, limit: WatchCopyStore.limit)
    }
    archive = loaded
    document = candidate
  }
  func select(_ id: String) throws {
    guard let entry = archive.entries.first(where: { $0.copy.descriptor.id == id }) else {
      throw SourceFailure.metadata
    }
    let candidate = ReadOnlyDocument()
    try candidate.importCopy(entry.copy.document, limit: WatchCopyStore.limit)
    var next = archive
    next.selected = id
    try persist(next)
    archive = next
    document = candidate
  }
  func remove(_ id: String) throws {
    var next = archive
    next.entries.removeAll { $0.copy.descriptor.id == id }
    if next.selected == id { next.selected = nil }
    try persist(next)
    archive = next
    if next.selected == nil { document = ReadOnlyDocument() }
  }
  func accept(_ copy: SourceCopy, source: String, select: Bool) throws {
    try copy.validate()
    if let current = archive.entries.first(where: { $0.copy.descriptor.id == copy.descriptor.id }) {
      let decision = try compareSource(current.copy.descriptor, copy.descriptor)
      guard decision == 0 || decision == 1 else { throw SourceFailure.rejected }
    }
    let candidate = ReadOnlyDocument()
    try candidate.importCopy(copy.document, limit: WatchCopyStore.limit)
    var next = archive
    next.entries.removeAll { $0.copy.descriptor.id == copy.descriptor.id }
    next.entries.append(Entry(copy: copy, receivedAt: Date().timeIntervalSince1970, source: source))
    guard next.entries.count <= 8 else { throw SourceFailure.oversized }
    // Total encoded cache bound also prevents unbounded cumulative storage.
    if select || next.selected == nil { next.selected = copy.descriptor.id }
    try persist(next)
    archive = next
    if next.selected == copy.descriptor.id { document = candidate }
  }
  private func persist(_ value: Archive) throws {
    let bytes = try JSONEncoder().encode(value)
    guard bytes.count <= 48 * 1024 * 1024 else { throw SourceFailure.oversized }
    try FileManager.default.createDirectory(
      at: url.deletingLastPathComponent(), withIntermediateDirectories: true)
    try bytes.write(to: url, options: .atomic)
  }
  func migrate(_ legacy: WatchCopyStore) throws {
    guard !FileManager.default.fileExists(atPath: url.path), archive.entries.isEmpty,
      let metadata = legacy.metadata
    else { return }
    let hash = SourceDescriptor.hash(legacy.document.source)
    let descriptor = SourceDescriptor(
      namespaceID: "legacy-phone", documentID: hash, revisionID: hash, parentRevisionID: "",
      contentSHA256: hash, serverGeneration: 0, kind: 0, title: metadata.title)
    try accept(
      SourceCopy(descriptor: descriptor, document: legacy.document.source),
      source: "Migrated iPhone copy", select: true)
  }
}
