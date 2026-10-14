import Foundation

@MainActor enum ViewerReplay {
  static func run() throws {
    let cancellation = NSError(domain: NSURLErrorDomain, code: NSURLErrorCancelled)
    var rejectedHTTP = BoundedDownload(limit: 3)
    precondition(!rejectedHTTP.response(status: 404, expectedLength: -1))
    if case .failure(let reason) = rejectedHTTP.result(transportError: cancellation) { precondition(reason as? DownloadFailure == .http(404)) } else { preconditionFailure("HTTP rejection lost") }
    var streaming = BoundedDownload(limit: 3)
    precondition(streaming.response(status: 200, expectedLength: -1))
    precondition(streaming.append(Data([1, 2])) && !streaming.append(Data([3, 4])))
    precondition(streaming.bytes == Data([1, 2]) && streaming.failure == .oversized)
    if case .failure(let reason) = streaming.result(transportError: cancellation) { precondition(reason as? DownloadFailure == .oversized) } else { preconditionFailure("Size rejection lost") }
    var declared = BoundedDownload(limit: 3)
    precondition(!declared.response(status: 200, expectedLength: 4) && declared.failure == .oversized)
    var redirect = BoundedDownload(limit: 3)
    precondition(!redirect.redirect(to: URL(string: "http://example.test/copy")))
    precondition(!redirect.response(status: 302, expectedLength: 0) && redirect.failure == .insecureRedirect)

    guard let sample = Bundle.main.url(forResource: "Sample", withExtension: "nsketch") else { throw ViewerError.invalid }
    let bytes = try boundedRead(sample, limit: 8 * 1024 * 1024)
    let document = ReadOnlyDocument(); try document.importCopy(bytes, limit: bytes.count)
    precondition(document.strokeCount > 0 && !document.paths.isEmpty)
    var navigation = ViewerNavigation(); navigation.magnify(2); navigation.pan = CGSize(width: 40, height: -20)
    _ = navigation.transform(bounds: document.bounds, size: CGSize(width: 200, height: 200)); navigation.fit()
    precondition(document.source == bytes && navigation.zoom == 1)
    #if VIEWER_REPLAY
    precondition(document.verifyReadOnlySnapshot())
    #endif
    do { try document.importCopy(Data("corrupt".utf8), limit: 100); preconditionFailure("Invalid accepted") } catch {}
    precondition(document.source == bytes)
    do { try document.importCopy(bytes, limit: bytes.count - 1); preconditionFailure("Oversized accepted") } catch {}
    precondition(document.source == bytes)
    let cache = FileManager.default.temporaryDirectory.appendingPathComponent(UUID().uuidString)
    defer { try? FileManager.default.removeItem(at: cache) }
    let store = WatchCopyStore(cache: cache)
    let metadata = try WatchCopyMetadata(["protocolVersion": 1, "sentAt": 100.0, "title": "Replay", "contentType": "dev.nativesketch.document"])
    try store.receive(bytes, metadata: metadata)
    let saved = try Data(contentsOf: cache)
    do { try store.receive(Data("bad".utf8), metadata: WatchCopyMetadata(["protocolVersion": 1, "sentAt": 200.0, "title": "Bad", "contentType": "dev.nativesketch.document"])); preconditionFailure("Invalid accepted") } catch {}
    do { try store.receive(bytes, metadata: metadata); preconditionFailure("Stale accepted") } catch {}
    let retained = try Data(contentsOf: cache)
    precondition(retained == saved)
    let oversized = Data(repeating: 0, count: WatchCopyStore.limit + 1)
    do { try store.receive(oversized, metadata: WatchCopyMetadata(["protocolVersion": 1, "sentAt": 300.0, "title": "Large", "contentType": "dev.nativesketch.document"])); preconditionFailure("Oversized accepted") } catch {}
    precondition(store.document.source == bytes && store.metadata?.sentAt == 100)
    do { _ = try boundedRead(cache, limit: 1); preconditionFailure("Unbounded read") } catch {}
    do { _ = try WatchCopyMetadata(["protocolVersion": 1, "sentAt": Double.nan, "title": "Invalid", "contentType": "dev.nativesketch.document"]); preconditionFailure("Invalid time") } catch {}
    let restored = WatchCopyStore(cache: cache); try restored.restore()
    precondition(restored.document.source == bytes && restored.metadata?.sentAt == 100)
    try FileManager.default.removeItem(at: cache)
    try FileManager.default.createDirectory(at: cache, withIntermediateDirectories: false)
    do { try store.receive(bytes, metadata: WatchCopyMetadata(["protocolVersion": 1, "sentAt": 400.0, "title": "Cannot write", "contentType": "dev.nativesketch.document"])); preconditionFailure("Failed write accepted") } catch {}
    precondition(store.document.source == bytes && store.metadata?.sentAt == 100)
  }
}
