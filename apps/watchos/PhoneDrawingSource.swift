import Foundation
import WatchConnectivity

@MainActor final class PhoneDrawingSource: NSObject, DrawingSource, WCSessionDelegate {
  private struct Pending {
    let resume: (Result<Data, Error>) -> Void
    let deadline: Task<Void, Never>
  }
  private var requests: [String: Pending] = [:]
  var pushed: (SourceCopy) -> Void = { _ in }
  var legacy: (Data, WatchCopyMetadata) -> Void = { _, _ in }
  var report: (String) -> Void = { _ in }
  override init() {
    super.init()
    if WCSession.isSupported() {
      WCSession.default.delegate = self
      WCSession.default.activate()
    }
  }
  func list() async throws -> [SourceDescriptor] {
    var result: [SourceDescriptor] = []
    for page in 0..<8 {
      let data = try await request(operation: "list", descriptor: nil, offset: page * 16)
      let items = try JSONDecoder().decode([SourceDescriptor].self, from: data)
      guard items.count <= 16 else { throw SourceFailure.oversized }
      for item in items { try item.validate() }
      result += items
      if items.count < 16 { break }
    }
    return result
  }
  func fetch(_ descriptor: SourceDescriptor) async throws -> SourceCopy {
    let data = try await request(operation: "fetch", descriptor: descriptor)
    let copy = try JSONDecoder().decode(SourceCopy.self, from: data)
    try copy.validate()
    return copy
  }
  private func request(operation: String, descriptor: SourceDescriptor?, offset: Int = 0)
    async throws -> Data
  {
    guard WCSession.isSupported(), WCSession.default.activationState == .activated,
      WCSession.default.isReachable
    else { throw SourceFailure.unavailable }
    let id = UUID().uuidString
    var message: [String: Any] = [
      "sourceProtocol": 2, "operation": operation, "requestID": id, "offset": offset,
    ]
    if let descriptor { message["descriptor"] = try JSONEncoder().encode(descriptor) }
    return try await withCheckedThrowingContinuation { continuation in
      let deadline = Task { @MainActor [weak self] in
        do { try await Task.sleep(for: .seconds(4)) } catch { return }
        self?.finish(id, .failure(SourceFailure.timeout))
      }
      requests[id] = Pending(resume: { continuation.resume(with: $0) }, deadline: deadline)
      let callbacks = PhoneMessageCallbacks(requestID: id) { [weak self] result in
        self?.finish(id, result)
      }
      WCSession.default.sendMessage(
        message, replyHandler: callbacks.reply, errorHandler: callbacks.error)
    }
  }
  private func finish(_ id: String, _ result: Result<Data, Error>) {
    guard let request = requests.removeValue(forKey: id) else { return }
    request.deadline.cancel()
    request.resume(result)
  }
  nonisolated func session(
    _ session: WCSession, activationDidCompleteWith activationState: WCSessionActivationState,
    error: Error?
  ) {}
  nonisolated func session(_ session: WCSession, didReceive file: WCSessionFile) {
    let retained = FileManager.default.temporaryDirectory.appendingPathComponent(UUID().uuidString)
    let version = file.metadata?["sourceProtocol"] as? Int
    let requestID = file.metadata?["requestID"] as? String
    let oldMetadata = try? WatchCopyMetadata(file.metadata ?? [:])
    do {
      // Retain the system-owned URL synchronously before the callback returns.
      try FileManager.default.moveItem(at: file.fileURL, to: retained)
      Task { @MainActor in
        defer { try? FileManager.default.removeItem(at: retained) }
        do {
          let data = try boundedRead(retained, limit: 12 * 1024 * 1024)
          if version == 2 {
            let copy = try JSONDecoder().decode(SourceCopy.self, from: data)
            try copy.validate()
            if let requestID {
              self.finish(requestID, .success(data))
            }  // Expired replies are ignored, never treated as pushes.
            else {
              self.pushed(copy)
            }
          } else if let oldMetadata {
            self.legacy(data, oldMetadata)
          } else {
            throw SourceFailure.metadata
          }
        } catch {
          if let requestID {
            self.finish(requestID, .failure(error))
          } else {
            self.report(error.localizedDescription)
          }
        }
      }
    } catch {
      Task { @MainActor in
        self.report("Could not retain the incoming copy. Previous drawings remain available.")
      }
    }
  }
}
