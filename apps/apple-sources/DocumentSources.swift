import Foundation
import CryptoKit

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
      guard !value.isEmpty, value.utf8.count <= 256, !value.unicodeScalars.contains(where: { CharacterSet.controlCharacters.contains($0) }) else { throw SourceFailure.metadata }
    }
    guard parentRevisionID.utf8.count <= 256, !parentRevisionID.unicodeScalars.contains(where: { CharacterSet.controlCharacters.contains($0) }), contentSHA256.count == 64, contentSHA256.utf8.allSatisfy({ (48...57).contains($0) || (97...102).contains($0) }), !title.isEmpty, title.utf8.count <= 2048, kind <= 1, kind == 0 || serverGeneration > 0 else { throw SourceFailure.metadata }
  }
  static func hash(_ data: Data) -> String { SHA256.hash(data: data).map { String(format: "%02x", $0) }.joined() }
}
struct SourceCopy: Codable, Sendable {
  let descriptor: SourceDescriptor
  let document: Data
  func validate() throws {
    try descriptor.validate()
    guard document.count <= 8 * 1024 * 1024 else { throw SourceFailure.oversized }
    guard SourceDescriptor.hash(document) == descriptor.contentSHA256 else { throw SourceFailure.integrity }
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
@MainActor final class SharedDrawingCatalog {
  let directory: URL
  init(directory: URL) { self.directory = directory }
  func list() throws -> [SourceDescriptor] {
    guard FileManager.default.fileExists(atPath: directory.path) else { return [] }
    let urls = try FileManager.default.contentsOfDirectory(at: directory, includingPropertiesForKeys: nil).filter { $0.pathExtension == "json" }.prefix(128)
    return urls.compactMap { try? load($0).descriptor }.sorted { $0.title.localizedStandardCompare($1.title) == .orderedAscending }
  }
  func share(_ data: Data, title: String, documentID: String) throws -> SourceCopy {
    let hash = SourceDescriptor.hash(data)
    let url = directory.appendingPathComponent(SourceDescriptor.hash(Data(documentID.utf8)) + ".json")
    let previous = try? load(url)
    if let previous, previous.descriptor.contentSHA256 == hash { return previous }
    let descriptor = SourceDescriptor(namespaceID: "phone-local", documentID: documentID, revisionID: UUID().uuidString, parentRevisionID: previous?.descriptor.revisionID ?? "", contentSHA256: hash, serverGeneration: 0, kind: 0, title: String(title.prefix(512)))
    let copy = SourceCopy(descriptor: descriptor, document: data); try copy.validate()
    try FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true)
    if !FileManager.default.fileExists(atPath: url.path), try list().count >= 128 { throw SourceFailure.oversized }
    try JSONEncoder().encode(copy).write(to: url, options: .atomic)
    return copy
  }
  func fetch(_ requested: SourceDescriptor) throws -> SourceCopy {
    try requested.validate()
    guard requested.namespaceID == "phone-local" else { throw SourceFailure.metadata }
    let copy = try load(directory.appendingPathComponent(SourceDescriptor.hash(Data(requested.documentID.utf8)) + ".json"))
    guard copy.descriptor.id == requested.id else { throw SourceFailure.rejected }; return copy
  }
  private func load(_ url: URL) throws -> SourceCopy {
    let size = try url.resourceValues(forKeys: [.fileSizeKey]).fileSize ?? Int.max
    guard size <= 12 * 1024 * 1024 else { throw SourceFailure.oversized }
    let copy = try JSONDecoder().decode(SourceCopy.self, from: Data(contentsOf: url)); try copy.validate(); return copy
  }
}
// Serial delegate queue exclusively owns response bytes and errors.
final class SourceHTTPTransfer: NSObject, URLSessionDataDelegate, @unchecked Sendable {
  private var bytes = Data()
  private var failure: Error?
  private let limit: Int
  private let origin: URL
  private let completion: @Sendable (Result<Data, Error>) -> Void
  init(origin: URL, limit: Int, completion: @escaping @Sendable (Result<Data, Error>) -> Void) { self.origin = origin; self.limit = limit; self.completion = completion }
  func urlSession(_ session: URLSession, task: URLSessionTask, willPerformHTTPRedirection response: HTTPURLResponse, newRequest request: URLRequest, completionHandler: @escaping @Sendable (URLRequest?) -> Void) {
    guard let url = request.url, url.scheme == "https", url.host == origin.host, url.port == origin.port else { failure = SourceFailure.http; completionHandler(nil); return }
    completionHandler(request)
  }
  func urlSession(_ session: URLSession, dataTask: URLSessionDataTask, didReceive response: URLResponse, completionHandler: @escaping @Sendable (URLSession.ResponseDisposition) -> Void) {
    guard let http = response as? HTTPURLResponse, (200...299).contains(http.statusCode), response.expectedContentLength <= limit else { failure = SourceFailure.http; completionHandler(.cancel); return }
    completionHandler(.allow)
  }
  func urlSession(_ session: URLSession, dataTask: URLSessionDataTask, didReceive data: Data) {
    guard data.count <= limit, bytes.count <= limit - data.count else { failure = SourceFailure.oversized; dataTask.cancel(); return }
    bytes.append(data)
  }
  func urlSession(_ session: URLSession, task: URLSessionTask, didCompleteWithError error: Error?) {
    completion((failure ?? error).map { .failure($0) } ?? .success(bytes)); session.finishTasksAndInvalidate()
  }
  static func get(_ url: URL, limit: Int, configuration: URLSessionConfiguration = .ephemeral) async throws -> Data {
    let cancellation = HTTPTaskCancellation()
    return try await withTaskCancellationHandler {
      try await withCheckedThrowingContinuation { continuation in
        let delegate = SourceHTTPTransfer(origin: url, limit: limit) { continuation.resume(with: $0) }
        let config = configuration; config.timeoutIntervalForRequest = 15; config.timeoutIntervalForResource = 20
        config.httpCookieStorage = nil; config.urlCredentialStorage = nil; config.urlCache = nil
        let queue = OperationQueue(); queue.maxConcurrentOperationCount = 1
        let task = URLSession(configuration: config, delegate: delegate, delegateQueue: queue).dataTask(with: url)
        cancellation.install(task); task.resume()
      }
    } onCancel: { cancellation.cancel() }
  }

}
@MainActor final class ServerDrawingSource: DrawingSource {
  let base: URL?
  init(base: URL?) {
    if let base, base.scheme == "https", base.host != nil, base.user == nil, base.password == nil, base.query == nil, base.fragment == nil { self.base = base } else { self.base = nil }
  }
  func list() async throws -> [SourceDescriptor] {
    guard let base else { throw SourceFailure.unconfigured }
    let data = try await SourceHTTPTransfer.get(base.appendingPathComponent("drawings"), limit: 256 * 1024)
    let result = try JSONDecoder().decode([SourceDescriptor].self, from: data)
    guard result.count <= 128 else { throw SourceFailure.oversized }
    for item in result { try item.validate(); guard !item.phoneOnly else { throw SourceFailure.metadata } }
    return result
  }
  func fetch(_ descriptor: SourceDescriptor) async throws -> SourceCopy {
    guard !descriptor.phoneOnly else { throw SourceFailure.phoneOnly }
    try descriptor.validate()
    guard let base else { throw SourceFailure.unconfigured }
    // Escape each ID as exactly one path component, including slash and percent.
    let safe = CharacterSet.alphanumerics.union(CharacterSet(charactersIn: "-_"))
    guard let id = descriptor.documentID.addingPercentEncoding(withAllowedCharacters: safe), var parts = URLComponents(url: base.appendingPathComponent("drawings"), resolvingAgainstBaseURL: false) else { throw SourceFailure.metadata }
    parts.percentEncodedPath += "/" + id
    parts.queryItems = [URLQueryItem(name: "namespace", value: descriptor.namespaceID), URLQueryItem(name: "revision", value: descriptor.revisionID)]
    guard let url = parts.url else { throw SourceFailure.metadata }
    let data = try await SourceHTTPTransfer.get(url, limit: 8 * 1024 * 1024)
    let copy = SourceCopy(descriptor: descriptor, document: data); try copy.validate(); return copy
  }
}

private final class HTTPTaskCancellation: @unchecked Sendable {
  private let lock = NSLock()
  private var task: URLSessionTask?
  private var cancelled = false
  func install(_ task: URLSessionTask) { lock.lock(); self.task = task; let stop = cancelled; lock.unlock(); if stop { task.cancel() } }
  func cancel() { lock.lock(); cancelled = true; let current = task; lock.unlock(); current?.cancel() }
}
