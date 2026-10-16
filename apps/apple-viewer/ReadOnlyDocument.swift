import CSketchFFI
import CoreGraphics
import Foundation

enum ViewerError: Error, LocalizedError {
  case invalid, oversized, stale, metadata
  var errorDescription: String? {
    switch self {
    case .invalid: "This is not a supported Native Sketch document."
    case .oversized: "The document exceeds this viewer's size limit."
    case .stale: "An older copy was ignored."
    case .metadata: "This transfer has unsupported metadata."
    }
  }
}

// Deliberately exposes no editing, selection, history, saving, or raw-handle API.
@MainActor final class ReadOnlyDocument {
  private var handle: OpaquePointer?
  private(set) var source = Data()
  private(set) var paths: [(CGPath, CGColor)] = []
  var bounds: CGRect { paths.reduce(CGRect.null) { $0.union($1.0.boundingBoxOfPath) } }
  var strokeCount: UInt64 {
    var state = SketchState()
    return sketch_editor_state(handle, &state) == 0 ? state.stroke_count : 0
  }
  init() { precondition(sketch_abi_version() == 2 && sketch_editor_create(&handle) == 0) }
  isolated deinit { sketch_editor_free(handle) }
  #if DEBUG
    // Verification only: no caller receives mutable engine access or exported bytes.
    func verifyReadOnlySnapshot() -> Bool {
      var count = 0
      var revision: UInt64 = 0
      guard sketch_document_export(handle, nil, 0, &count, &revision) == 0 else { return false }
      var bytes = Data(count: count)
      let code = bytes.withUnsafeMutableBytes {
        sketch_document_export(
          handle, $0.bindMemory(to: UInt8.self).baseAddress, count, &count, &revision)
      }
      return code == 0 && bytes == source
    }
  #endif
  func importCopy(_ data: Data, limit: Int) throws {
    guard data.count <= limit else { throw ViewerError.oversized }
    // Validate in a separate handle so even a Rust panic cannot destroy the last good copy.
    var candidate: OpaquePointer?
    guard sketch_editor_create(&candidate) == 0 else { throw ViewerError.invalid }
    defer { sketch_editor_free(candidate) }
    guard
      data.withUnsafeBytes({
        sketch_document_import(candidate, $0.bindMemory(to: UInt8.self).baseAddress, data.count)
      }) == 0
    else { throw ViewerError.invalid }
    var count = 0
    guard sketch_render_copy(candidate, nil, 0, &count) == 0 else { throw ViewerError.invalid }
    var commands = Array(repeating: SketchPathCommand(), count: count)
    guard
      commands.withUnsafeMutableBufferPointer({
        sketch_render_copy(candidate, $0.baseAddress, count, &count)
      }) == 0
    else { throw ViewerError.invalid }
    var result: [(CGPath, CGColor)] = []
    var id: UInt64?
    var path = CGMutablePath()
    var color = CGColor(gray: 0, alpha: 1)
    for c in commands {
      if id != c.stroke_id {
        if id != nil { result.append((path, color)) }
        id = c.stroke_id
        path = CGMutablePath()
        color = CGColor(srgbRed: c.red, green: c.green, blue: c.blue, alpha: c.alpha)
      }
      switch c.verb {
      case 0: path.move(to: CGPoint(x: c.x1, y: c.y1))
      case 1: path.addLine(to: CGPoint(x: c.x1, y: c.y1))
      case 2:
        path.addCurve(
          to: CGPoint(x: c.x3, y: c.y3), control1: CGPoint(x: c.x1, y: c.y1),
          control2: CGPoint(x: c.x2, y: c.y2))
      default: path.closeSubpath()
      }
    }
    if id != nil { result.append((path, color)) }
    swap(&handle, &candidate)
    paths = result
    source = data
  }
}

struct ViewerNavigation {
  var zoom = 1.0
  var pan = CGSize.zero
  mutating func fit() {
    zoom = 1
    pan = .zero
  }
  mutating func magnify(_ factor: Double) { zoom = min(16, max(0.25, zoom * factor)) }
  func transform(bounds: CGRect, size: CGSize) -> CGAffineTransform {
    guard !bounds.isNull, bounds.width > 0, bounds.height > 0 else { return .identity }
    let scale =
      min(max(1, size.width - 24) / bounds.width, max(1, size.height - 24) / bounds.height) * zoom
    return CGAffineTransform(
      translationX: size.width / 2 + pan.width, y: size.height / 2 + pan.height
    )
    .scaledBy(x: scale, y: scale).translatedBy(x: -bounds.midX, y: -bounds.midY)
  }
}

struct WatchCopyMetadata: Codable, Sendable {
  let protocolVersion: Int
  let sentAt: Double
  let title: String
  let contentType: String
  init(_ values: [String: Any]) throws {
    guard let version = values["protocolVersion"] as? Int, version == 1,
      let timestamp = values["sentAt"] as? Double, timestamp.isFinite, timestamp > 0,
      let title = values["title"] as? String, !title.isEmpty, title.count <= 512,
      values["contentType"] as? String == "dev.nativesketch.document"
    else { throw ViewerError.metadata }
    protocolVersion = version
    sentAt = timestamp
    self.title = title
    contentType = "dev.nativesketch.document"
  }
}

// One atomic envelope keeps metadata and document together across interruptions.
@MainActor final class WatchCopyStore {
  static let limit = 8 * 1024 * 1024
  struct Envelope: Codable {
    let metadata: WatchCopyMetadata
    let document: Data
    let receivedAt: Double
  }
  private(set) var document = ReadOnlyDocument()
  let cache: URL
  private(set) var metadata: WatchCopyMetadata?
  private(set) var receivedAt: Double?
  init(cache: URL) { self.cache = cache }
  func restore() throws {
    guard FileManager.default.fileExists(atPath: cache.path) else { return }
    let bytes = try boundedRead(cache, limit: Self.limit * 2)
    let envelope = try JSONDecoder().decode(Envelope.self, from: bytes)
    _ = try WatchCopyMetadata([
      "protocolVersion": envelope.metadata.protocolVersion, "sentAt": envelope.metadata.sentAt,
      "title": envelope.metadata.title, "contentType": envelope.metadata.contentType,
    ])
    guard envelope.receivedAt.isFinite, envelope.receivedAt > 0 else { throw ViewerError.metadata }
    try document.importCopy(envelope.document, limit: Self.limit)
    metadata = envelope.metadata
    receivedAt = envelope.receivedAt
  }
  func receive(_ data: Data, metadata next: WatchCopyMetadata) throws {
    guard next.sentAt > (metadata?.sentAt ?? 0) else { throw ViewerError.stale }
    let candidate = ReadOnlyDocument()
    try candidate.importCopy(data, limit: Self.limit)
    let received = Date().timeIntervalSince1970
    let bytes = try JSONEncoder().encode(
      Envelope(metadata: next, document: data, receivedAt: received))
    try FileManager.default.createDirectory(
      at: cache.deletingLastPathComponent(), withIntermediateDirectories: true)
    try bytes.write(to: cache, options: .atomic)
    document = candidate
    metadata = next
    receivedAt = received
  }
}

func boundedRead(_ url: URL, limit: Int) throws -> Data {
  guard let stream = InputStream(url: url) else { throw ViewerError.invalid }
  stream.open()
  defer { stream.close() }
  var result = Data()
  var buffer = [UInt8](repeating: 0, count: 65536)
  while true {
    let count = stream.read(&buffer, maxLength: buffer.count)
    guard count >= 0 else { throw stream.streamError ?? ViewerError.invalid }
    if count == 0 { return result }
    guard result.count <= limit - count else { throw ViewerError.oversized }
    result.append(buffer, count: count)
  }
}
