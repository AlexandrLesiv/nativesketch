#if os(iOS)
  import Foundation
  import WatchConnectivity

  private final class WatchReply: @unchecked Sendable {
    let call: ([String: Any]) -> Void
    init(_ call: @escaping ([String: Any]) -> Void) { self.call = call }
  }

  // Only explicit user actions enqueue a transfer; a copy never acknowledges a save.
  @MainActor final class WatchCopySender: NSObject, WCSessionDelegate {
    var documentID = UUID().uuidString
    private let catalog = SharedDrawingCatalog(
      directory: FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask)[0]
        .appendingPathComponent("WatchShared"))
    private var session: WCSession?
    private var pending: (Data, String, String, (String) -> Void)?
    private var completions: [URL: (String) -> Void] = [:]
    func activate() {
      guard WCSession.isSupported() else { return }
      let session = WCSession.default
      self.session = session
      session.delegate = self
      session.activate()
    }
    func send(_ data: Data, title: String, report: @escaping (String) -> Void) {
      guard data.count <= 8 * 1024 * 1024 else {
        report("This drawing exceeds the 8 MiB Watch copy limit. Send a smaller drawing.")
        return
      }
      guard WCSession.isSupported() else {
        report("Apple Watch transfer is unavailable on this device.")
        return
      }
      guard pending == nil else {
        report("Watch connection is still activating. Try again shortly.")
        return
      }
      let session = self.session ?? WCSession.default
      self.session = session
      session.delegate = self
      if session.activationState != .activated {
        pending = (data, title, documentID, report)
        session.activate()
        return
      }
      enqueue(data, title: title, documentID: documentID, report: report)
    }
    private func enqueue(
      _ data: Data, title: String, documentID: String, report: @escaping (String) -> Void
    ) {
      guard let session, session.isPaired else {
        report("Pair an Apple Watch with this iPhone first.")
        return
      }
      guard session.isWatchAppInstalled else {
        report("Install Native Sketch on your paired Apple Watch first.")
        return
      }
      do {
        let directory = FileManager.default.urls(
          for: .applicationSupportDirectory, in: .userDomainMask)[0]
          .appendingPathComponent("WatchOutgoing", isDirectory: true)
        try FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true)
        let url = directory.appendingPathComponent(UUID().uuidString + ".nsketch")
        let copy = try catalog.share(data, title: title, documentID: documentID)
        try JSONEncoder().encode(copy).write(to: url, options: .atomic)
        completions[url] = report
        session.transferFile(
          url,
          metadata: [
            "sourceProtocol": 2
          ])
        report(
          "Copy queued for Apple Watch. Delivery can occur later when the devices connect. Sending a copy does not change this document’s saved status."
        )
      } catch { report("Could not queue the copy: \(error.localizedDescription)") }
    }
    nonisolated func session(
      _ session: WCSession, activationDidCompleteWith activationState: WCSessionActivationState,
      error: Error?
    ) {
      let message = error?.localizedDescription
      let activated = activationState == .activated
      Task { @MainActor in
        guard let pending = self.pending else { return }
        self.pending = nil
        if activated {
          self.enqueue(pending.0, title: pending.1, documentID: pending.2, report: pending.3)
        } else {
          pending.3("Could not connect to Apple Watch: \(message ?? "try again after pairing")")
        }
      }
    }
    nonisolated func session(
      _ session: WCSession, didReceiveMessage message: [String: Any],
      replyHandler: @escaping ([String: Any]) -> Void
    ) {
      let reply = WatchReply(replyHandler)
      let id = message["requestID"] as? String
      let version = message["sourceProtocol"] as? Int
      let operation = message["operation"] as? String
      let offset = message["offset"] as? Int ?? 0
      let encoded = message["descriptor"] as? Data
      Task { @MainActor in
        guard version == 2, let id, UUID(uuidString: id) != nil else {
          reply.call(["error": "Invalid request"])
          return
        }
        do {
          if operation == "list" {
            guard offset >= 0, offset < 128, offset % 16 == 0 else { throw SourceFailure.metadata }
            let items = Array(try self.catalog.list().dropFirst(offset).prefix(16))
            let data = try JSONEncoder().encode(items)
            guard data.count <= 48 * 1024 else { throw SourceFailure.oversized }
            reply.call(["requestID": id, "payload": data])
          } else if operation == "fetch", let encoded, encoded.count <= 8192 {
            let descriptor = try JSONDecoder().decode(SourceDescriptor.self, from: encoded)
            let copy = try self.catalog.fetch(descriptor)
            let directory = FileManager.default.urls(
              for: .applicationSupportDirectory, in: .userDomainMask)[0].appendingPathComponent(
                "WatchOutgoing")
            try FileManager.default.createDirectory(
              at: directory, withIntermediateDirectories: true)
            let url = directory.appendingPathComponent(UUID().uuidString + ".json")
            try JSONEncoder().encode(copy).write(to: url, options: .atomic)
            WCSession.default.transferFile(url, metadata: ["sourceProtocol": 2, "requestID": id])
            reply.call(["requestID": id, "queued": true])
          } else {
            throw SourceFailure.metadata
          }
        } catch { reply.call(["requestID": id, "error": "Shared copy unavailable"]) }
      }
    }
    nonisolated func sessionDidBecomeInactive(_ session: WCSession) {}
    nonisolated func sessionDidDeactivate(_ session: WCSession) { session.activate() }
    nonisolated func session(
      _ session: WCSession, didFinish fileTransfer: WCSessionFileTransfer, error: Error?
    ) {
      let url = fileTransfer.file.fileURL
      let message = error?.localizedDescription
      Task { @MainActor in
        // WatchConnectivity has finished reading this URL. Never delete queued files earlier.
        try? FileManager.default.removeItem(at: url)
        let report = self.completions.removeValue(forKey: url)
        if let message {
          report?("Watch transfer failed: \(message). Use Send Copy to Apple Watch to retry.")
        }
        // Successful transport does not prove that the receiving app imported the file.
      }
    }
  }
#endif
