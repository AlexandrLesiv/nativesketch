import Foundation
import CSketchFFI

@MainActor final class WatchLibraryStore {
  struct Entry: Codable { let copy: SourceCopy; let receivedAt: Double; let source: String }
  struct Archive: Codable { var selected: String?; var entries: [Entry] }
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
      guard entry.receivedAt.isFinite, entry.receivedAt > 0, identities.insert(entry.copy.descriptor.id).inserted else { throw SourceFailure.metadata }
    }
    guard loaded.selected == nil || identities.contains(loaded.selected!) else { throw SourceFailure.metadata }
    let candidate = ReadOnlyDocument()
    if let entry = loaded.entries.first(where: { $0.copy.descriptor.id == loaded.selected }) { try candidate.importCopy(entry.copy.document, limit: WatchCopyStore.limit) }
    archive = loaded; document = candidate
  }
  func select(_ id: String) throws {
    guard let entry = archive.entries.first(where: { $0.copy.descriptor.id == id }) else { throw SourceFailure.metadata }
    let candidate = ReadOnlyDocument(); try candidate.importCopy(entry.copy.document, limit: WatchCopyStore.limit)
    var next = archive; next.selected = id; try persist(next); archive = next; document = candidate
  }
  func remove(_ id: String) throws {
    var next = archive; next.entries.removeAll { $0.copy.descriptor.id == id }
    if next.selected == id { next.selected = nil }
    try persist(next); archive = next
    if next.selected == nil { document = ReadOnlyDocument() }
  }
  func accept(_ copy: SourceCopy, source: String, select: Bool) throws {
    try copy.validate()
    if let current = archive.entries.first(where: { $0.copy.descriptor.id == copy.descriptor.id }) {
      let decision = try compareSource(current.copy.descriptor, copy.descriptor)
      guard decision == 0 || decision == 1 else { throw SourceFailure.rejected }
    }
    let candidate = ReadOnlyDocument(); try candidate.importCopy(copy.document, limit: WatchCopyStore.limit)
    var next = archive
    next.entries.removeAll { $0.copy.descriptor.id == copy.descriptor.id }
    next.entries.append(Entry(copy: copy, receivedAt: Date().timeIntervalSince1970, source: source))
    guard next.entries.count <= 8 else { throw SourceFailure.oversized }
    // Total encoded cache bound also prevents unbounded cumulative storage.
    if select || next.selected == nil { next.selected = copy.descriptor.id }
    try persist(next); archive = next
    if next.selected == copy.descriptor.id { document = candidate }
  }
  private func persist(_ value: Archive) throws {
    let bytes = try JSONEncoder().encode(value)
    guard bytes.count <= 48 * 1024 * 1024 else { throw SourceFailure.oversized }
    try FileManager.default.createDirectory(at: url.deletingLastPathComponent(), withIntermediateDirectories: true)
    try bytes.write(to: url, options: .atomic)
  }
  func migrate(_ legacy: WatchCopyStore) throws {
    guard archive.entries.isEmpty, let metadata = legacy.metadata else { return }
    let hash = SourceDescriptor.hash(legacy.document.source)
    let descriptor = SourceDescriptor(namespaceID: "legacy-phone", documentID: hash, revisionID: hash, parentRevisionID: "", contentSHA256: hash, serverGeneration: 0, kind: 0, title: metadata.title)
    try accept(SourceCopy(descriptor: descriptor, document: legacy.document.source), source: "Migrated iPhone copy", select: true)
  }
}

// Single owner of request generations. Late responses cannot update another selection.
@MainActor final class SourceCoordinator {
  let phone: any DrawingSource
  let server: any DrawingSource
  let store: WatchLibraryStore
  private var generation = 0
  var navigating = false
  private(set) var pending: (SourceCopy, String)?
  var changed: () -> Void = {}
  var report: (String) -> Void = { _ in }
  init(phone: any DrawingSource, server: any DrawingSource, store: WatchLibraryStore) { self.phone = phone; self.server = server; self.store = store }
  func cancel() { generation += 1; pending = nil }
  func list() async throws -> [SourceDescriptor] {
    do { return try await phone.list() } catch { return try await server.list() }
  }
  func refresh(_ descriptor: SourceDescriptor) async {
    generation += 1; let request = generation
    do {
      let copy: SourceCopy; let source: String
      do { copy = try await phone.fetch(descriptor); source = "iPhone" }
      catch {
        guard request == generation else { return }
        guard !descriptor.phoneOnly else { throw SourceFailure.phoneOnly }
        // Fetch catalog again to discover a newer server revision of exactly this identity.
        let catalog = try await server.list()
        guard let latest = catalog.first(where: { $0.id == descriptor.id }) else { throw SourceFailure.unavailable }
        copy = try await server.fetch(latest); source = "Server"
      }
      guard request == generation else { return }
      guard copy.descriptor.id == descriptor.id else { throw SourceFailure.rejected }
      try copy.validate()
      let decision = try compareSource(descriptor, copy.descriptor)
      guard decision == 0 || decision == 1 else { throw SourceFailure.rejected }
      if navigating, store.selected?.copy.descriptor.id == descriptor.id, store.selected?.copy.descriptor.contentSHA256 != copy.descriptor.contentSHA256 {
        try stage(copy, source: source); report("New version available. Use Apply Update in Details."); changed()
      } else { try store.accept(copy, source: source, select: true); pending = nil; report("Updated from " + source); changed() }
    } catch { if request == generation { report(error.localizedDescription); changed() } }
  }
  private func stage(_ copy: SourceCopy, source: String) throws {
    if let pending, pending.0.descriptor.id == copy.descriptor.id {
      let decision = try compareSource(pending.0.descriptor, copy.descriptor)
      guard decision == 0 || decision == 1 else { throw SourceFailure.rejected }
    }
    let candidate = ReadOnlyDocument()
    try candidate.importCopy(copy.document, limit: WatchCopyStore.limit)
    pending = (copy, source)
  }
  func apply() throws {
    guard let (copy, source) = pending, copy.descriptor.id == store.selected?.copy.descriptor.id else { pending = nil; return }
    try store.accept(copy, source: source, select: true); pending = nil; navigating = false; changed()
  }
  func pushed(_ copy: SourceCopy) throws {
    try copy.validate()
    // A push adds to the library; it never hijacks an existing selection.
    if copy.descriptor.id == store.selected?.copy.descriptor.id, navigating, store.selected?.copy.descriptor.contentSHA256 != copy.descriptor.contentSHA256 {
      if let current = store.selected { let decision = try compareSource(current.copy.descriptor, copy.descriptor); guard decision == 0 || decision == 1 else { throw SourceFailure.rejected } }
      try stage(copy, source: "iPhone"); report("New version available. Use Apply Update in Details.")
    } else { try store.accept(copy, source: "iPhone", select: store.selected == nil) }
    changed()
  }
}

private func withSourceVersion<T>(_ descriptor: SourceDescriptor, _ body: (UnsafePointer<SketchSourceVersion>) throws -> T) rethrows -> T {
  let values = [descriptor.namespaceID, descriptor.documentID, descriptor.revisionID, descriptor.parentRevisionID, descriptor.contentSHA256].map { Array($0.utf8) }
  return try values[0].withUnsafeBufferPointer { a in try values[1].withUnsafeBufferPointer { b in try values[2].withUnsafeBufferPointer { c in try values[3].withUnsafeBufferPointer { d in try values[4].withUnsafeBufferPointer { e in
    var version = SketchSourceVersion(namespace_id: SketchSourceText(bytes: a.baseAddress, length: a.count), document_id: SketchSourceText(bytes: b.baseAddress, length: b.count), revision_id: SketchSourceText(bytes: c.baseAddress, length: c.count), parent_revision_id: SketchSourceText(bytes: d.baseAddress, length: d.count), content_sha256: SketchSourceText(bytes: e.baseAddress, length: e.count), server_generation: descriptor.serverGeneration, kind: descriptor.kind)
    return try body(&version)
  } } } } }
}
func compareSource(_ current: SourceDescriptor, _ candidate: SourceDescriptor) throws -> UInt32 {
  try current.validate(); try candidate.validate()
  var decision: UInt32 = 0
  let status = withSourceVersion(current) { a in withSourceVersion(candidate) { b in sketch_source_compare(a, b, &decision) } }
  guard status == 0 else { throw SourceFailure.metadata }; return decision
}
