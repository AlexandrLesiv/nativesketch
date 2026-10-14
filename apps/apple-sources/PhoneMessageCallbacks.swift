import Foundation

// Constructed outside any actor context: WatchConnectivity invokes these callbacks
// on arbitrary utility queues. Parse only value types here, then explicitly hop.
struct PhoneMessageCallbacks: Sendable {
  let reply: @Sendable ([String: Any]) -> Void
  let error: @Sendable (Error) -> Void
  nonisolated init(requestID: String, receive: @escaping @MainActor @Sendable (Result<Data, Error>) -> Void) {
    reply = { response in
      let id = response["requestID"] as? String
      let payload = response["payload"] as? Data
      let queued = response["queued"] as? Bool == true
      let result: Result<Data, Error>?
      if id != requestID { result = .failure(SourceFailure.metadata) }
      else if let payload, payload.count <= 48 * 1024 { result = .success(payload) }
      else if !queued { result = .failure(SourceFailure.unavailable) }
      else { result = nil }
      if let result { Task { @MainActor in receive(result) } }
    }
    error = { _ in Task { @MainActor in receive(.failure(SourceFailure.unavailable)) } }
  }
}
