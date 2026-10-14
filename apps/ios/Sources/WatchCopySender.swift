#if os(iOS)
  import Foundation
  import WatchConnectivity

  // Only explicit user actions enqueue a transfer; a copy never acknowledges a save.
  @MainActor final class WatchCopySender: NSObject, WCSessionDelegate {
    private var session: WCSession?
    private var pending: (Data, String, (String) -> Void)?
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
        pending = (data, title, report)
        session.activate()
        return
      }
      enqueue(data, title: title, report: report)
    }
    private func enqueue(_ data: Data, title: String, report: @escaping (String) -> Void) {
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
        try data.write(to: url, options: .atomic)
        completions[url] = report
        session.transferFile(
          url,
          metadata: [
            "protocolVersion": 1, "sentAt": Date().timeIntervalSince1970,
            "title": title, "contentType": "dev.nativesketch.document",
          ])
        report(
          "Copy queued for Apple Watch. Delivery can occur later when the devices connect. Your document remains unsaved until you save it to Files."
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
          self.enqueue(pending.0, title: pending.1, report: pending.2)
        } else {
          pending.2("Could not connect to Apple Watch: \(message ?? "try again after pairing")")
        }
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
