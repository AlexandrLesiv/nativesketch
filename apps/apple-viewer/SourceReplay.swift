#if DEBUG
import Foundation
import CSketchFFI

@MainActor private final class StubDrawingSource: DrawingSource {
  var copies: [SourceCopy]
  var failure = false
  var delay: Duration = .zero
  var fetches = 0
  init(_ copies: [SourceCopy]) { self.copies = copies }
  func list() async throws -> [SourceDescriptor] { if failure { throw SourceFailure.unavailable }; return copies.map(\.descriptor) }
  func fetch(_ descriptor: SourceDescriptor) async throws -> SourceCopy {
    fetches += 1; if delay != .zero { try await Task.sleep(for: delay) }
    if failure { throw SourceFailure.unavailable }
    guard let copy = copies.first(where: { $0.descriptor.id == descriptor.id }) else { throw SourceFailure.unavailable }; return copy
  }
}
@MainActor enum SourceReplay {
  static func run() async throws {
    try await SourceHTTPReplay.run()
    guard let fixture = Bundle.main.url(forResource: "Sample", withExtension: "nsketch") else { throw SourceFailure.metadata }
    let data = try boundedRead(fixture, limit: WatchCopyStore.limit)
    let root = FileManager.default.temporaryDirectory.appendingPathComponent(UUID().uuidString)
    defer { try? FileManager.default.removeItem(at: root) }
    let hash = SourceDescriptor.hash(data)
    func copy(_ id: String, _ revision: String, _ generation: UInt64) -> SourceCopy {
      SourceCopy(descriptor: SourceDescriptor(namespaceID: "test", documentID: id, revisionID: revision, parentRevisionID: "", contentSHA256: hash, serverGeneration: generation, kind: 1, title: id), document: data)
    }
    let a = copy("a", "one", 1), newer = copy("a", "two", 2), b = copy("b", "one", 1)
    let store = WatchLibraryStore(url: root.appendingPathComponent("cache.json"))
    try store.accept(a, source: "fixture", select: true)
    let phone = StubDrawingSource([newer]), server = StubDrawingSource([newer])
    let coordinator = SourceCoordinator(phone: phone, server: server, store: store)
    await coordinator.refresh(a.descriptor)
    guard server.fetches == 0, store.selected?.copy.descriptor.revisionID == "two" else { throw SourceFailure.rejected }
    phone.failure = true
    await coordinator.refresh(newer.descriptor)
    guard server.fetches == 1 else { throw SourceFailure.rejected }
    // Wrong identity and lower generations never replace the selected document.
    do { try store.accept(a, source: "stale", select: true); throw SourceFailure.metadata } catch SourceFailure.rejected {}
    let third = copy("a", "three", 3); server.copies = [third]
    await coordinator.refresh(newer.descriptor)
    guard store.selected?.copy.descriptor.revisionID == "three" else { throw SourceFailure.rejected }
    let restored = WatchLibraryStore(url: root.appendingPathComponent("cache.json")); try restored.restore()
    guard restored.selected?.copy.descriptor.revisionID == "three" else { throw SourceFailure.rejected }
    phone.failure = false; phone.delay = .milliseconds(50)
    let slow = Task { @MainActor in await coordinator.refresh(newer.descriptor) }
    await Task.yield(); coordinator.cancel(); try store.accept(b, source: "fixture", select: true)
    await slow.value
    guard store.selected?.copy.descriptor.documentID == "b" else { throw SourceFailure.rejected }
    var corrupt = data; corrupt.append(0)
    do { try SourceCopy(descriptor: b.descriptor, document: corrupt).validate(); throw SourceFailure.metadata } catch SourceFailure.integrity {}
    // A changed but valid Rust document is staged while the viewer is navigating.
    var handle: OpaquePointer?; guard sketch_editor_create(&handle) == 0 else { throw SourceFailure.metadata }
    defer { sketch_editor_free(handle) }
    var count = 0; var revision: UInt64 = 0
    guard sketch_document_export(handle, nil, 0, &count, &revision) == 0 else { throw SourceFailure.metadata }
    var empty = Data(count: count)
    guard empty.withUnsafeMutableBytes({ sketch_document_export(handle, $0.bindMemory(to: UInt8.self).baseAddress, count, &count, &revision) }) == 0 else { throw SourceFailure.metadata }
    let changed = SourceCopy(descriptor: SourceDescriptor(namespaceID: "test", documentID: "b", revisionID: "two", parentRevisionID: "", contentSHA256: SourceDescriptor.hash(empty), serverGeneration: 2, kind: 1, title: "b"), document: empty)
    phone.delay = .zero; phone.copies = [changed]; coordinator.navigating = true
    await coordinator.refresh(b.descriptor)
    guard coordinator.pending != nil, store.document.source == data else { throw SourceFailure.rejected }
    let stagedNewer = SourceCopy(descriptor: SourceDescriptor(namespaceID: "test", documentID: "b", revisionID: "three", parentRevisionID: "", contentSHA256: SourceDescriptor.hash(empty), serverGeneration: 3, kind: 1, title: "b"), document: empty)
    try coordinator.pushed(stagedNewer)
    do { try coordinator.pushed(changed); throw SourceFailure.metadata } catch SourceFailure.rejected {}
    // Same-content metadata updates must also respect the pending newer revision.
    phone.copies = [copy("b", "two", 2)]
    await coordinator.refresh(b.descriptor)
    guard store.selected?.copy.descriptor.revisionID == "one" else { throw SourceFailure.rejected }
    guard coordinator.pending?.0.descriptor.revisionID == "three" else { throw SourceFailure.rejected }
    try coordinator.apply()
    guard coordinator.pending == nil, store.document.source == empty else { throw SourceFailure.rejected }
    let legacy = WatchCopyStore(cache: root.appendingPathComponent("legacy.json"))
    try legacy.receive(data, metadata: WatchCopyMetadata(["protocolVersion": 1, "sentAt": 1.0, "title": "Legacy", "contentType": "dev.nativesketch.document"]))
    let migrated = WatchLibraryStore(url: root.appendingPathComponent("migrated.json")); try migrated.migrate(legacy)
    guard migrated.selected?.copy.descriptor.phoneOnly == true, FileManager.default.fileExists(atPath: legacy.cache.path) else { throw SourceFailure.rejected }
    try migrated.remove(migrated.selected!.copy.descriptor.id)
    let emptyLibrary = WatchLibraryStore(url: migrated.url); try emptyLibrary.restore(); try emptyLibrary.migrate(legacy)
    guard emptyLibrary.archive.entries.isEmpty else { throw SourceFailure.rejected }
    let missing = ServerDrawingSource(base: nil)
    do { _ = try await missing.list(); throw SourceFailure.metadata } catch SourceFailure.unconfigured {}
    let catalog = SharedDrawingCatalog(directory: root.appendingPathComponent("shared"))
    let shared = try catalog.share(data, title: "Explicit", documentID: "stable-session")
    guard try catalog.fetch(shared.descriptor).document == data, try catalog.list().count == 1 else { throw SourceFailure.rejected }
    try store.remove(b.descriptor.id)
    guard store.selected == nil, store.document.source.isEmpty else { throw SourceFailure.rejected }
    print("SOURCE REPLAY: phone preference, fallback, stale rejection, cache restore, delayed generation, integrity, unconfigured server, explicit catalog, staged geometry, legacy migration PASS")
  }
}
#endif
