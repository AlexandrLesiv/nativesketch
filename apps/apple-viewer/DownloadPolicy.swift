import Foundation

enum DownloadFailure: Error, LocalizedError, Equatable {
  case http(Int)
  case oversized, insecureRedirect, invalidResponse
  var errorDescription: String? {
    switch self {
    case .http(let status): "The server returned HTTP \(status)."
    case .oversized: "This document exceeds the 64 MiB Apple TV limit."
    case .insecureRedirect: "The server redirected to a non-HTTPS address."
    case .invalidResponse: "The server did not return an HTTP document."
    }
  }
}

// Used only on the URLSession's serial delegate queue. The first rejection survives
// subsequent URLSession cancellation errors, so the user sees the actual cause.
struct BoundedDownload {
  let limit: Int
  private(set) var bytes = Data()
  private(set) var failure: DownloadFailure?
  mutating func redirect(to url: URL?) -> Bool {
    guard url?.scheme?.lowercased() == "https" else {
      reject(.insecureRedirect)
      return false
    }
    return failure == nil
  }
  mutating func response(status: Int?, expectedLength: Int64) -> Bool {
    guard let status else {
      reject(.invalidResponse)
      return false
    }
    guard (200...299).contains(status) else {
      reject(.http(status))
      return false
    }
    guard expectedLength <= Int64(limit) else {
      reject(.oversized)
      return false
    }
    return failure == nil
  }
  mutating func append(_ data: Data) -> Bool {
    guard failure == nil else { return false }
    guard data.count <= limit, bytes.count <= limit - data.count else {
      reject(.oversized)
      return false
    }
    bytes.append(data)
    return true
  }
  mutating private func reject(_ error: DownloadFailure) { if failure == nil { failure = error } }
  func result(transportError: Error?) -> Result<Data, Error> {
    if let failure { return .failure(failure) }
    if let transportError { return .failure(transportError) }
    return .success(bytes)
  }
}
