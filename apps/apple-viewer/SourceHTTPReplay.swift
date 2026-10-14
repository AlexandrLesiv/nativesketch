#if DEBUG
import Foundation

// Uses real URLSession delegate delivery; the only stub is the protocol transport.
private final class SourceFixtureProtocol: URLProtocol, @unchecked Sendable {
  override class func canInit(with request: URLRequest) -> Bool { true }
  override class func canonicalRequest(for request: URLRequest) -> URLRequest { request }
  override func startLoading() {
    guard let url = request.url else { return }
    if url.path == "/wait" { return }
    let status = url.path == "/status" ? 401 : 200
    let headers = url.path == "/declared" ? ["Content-Length": "999"] : [:]
    let response = HTTPURLResponse(url: url, statusCode: status, httpVersion: "HTTP/1.1", headerFields: headers)!
    client?.urlProtocol(self, didReceive: response, cacheStoragePolicy: .notAllowed)
    client?.urlProtocol(self, didLoad: Data([1,2,3]))
    if url.path == "/stream" { client?.urlProtocol(self, didLoad: Data([4,5,6])) }
    client?.urlProtocolDidFinishLoading(self)
  }
  override func stopLoading() {}
}
@MainActor enum SourceHTTPReplay {
  static func run() async throws {
    func config() -> URLSessionConfiguration { let c = URLSessionConfiguration.ephemeral; c.protocolClasses = [SourceFixtureProtocol.self]; return c }
    let base = URL(string: "https://fixture.invalid")!
    let good = try await SourceHTTPTransfer.get(base.appendingPathComponent("good"), limit: 4, configuration: config())
    guard good == Data([1,2,3]) else { throw SourceFailure.metadata }
    for path in ["status", "declared", "stream"] {
      var rejected = false
      do { _ = try await SourceHTTPTransfer.get(base.appendingPathComponent(path), limit: 4, configuration: config()) } catch { rejected = true }
      guard rejected else { throw SourceFailure.metadata }
    }
    let waiting = Task { try await SourceHTTPTransfer.get(base.appendingPathComponent("wait"), limit: 4, configuration: config()) }
    await Task.yield(); waiting.cancel()
    var cancelled = false
    do { _ = try await waiting.value } catch { cancelled = true }
    guard cancelled else { throw SourceFailure.metadata }
    // Exercise the production redirect delegate directly: URLProtocol redirect support
    // varies by SDK; no external server is required to verify origin rejection.
    let transfer = SourceHTTPTransfer(origin: base, limit: 4) { _ in }
    let session = URLSession(configuration: .ephemeral)
    let task = session.dataTask(with: base)
    let response = HTTPURLResponse(url: base, statusCode: 302, httpVersion: nil, headerFields: nil)!
    for address in ["http://fixture.invalid/x", "https://other.invalid/x"] {
      let allowed = await withCheckedContinuation { continuation in
        transfer.urlSession(session, task: task, willPerformHTTPRedirection: response, newRequest: URLRequest(url: URL(string: address)!)) { continuation.resume(returning: $0 != nil) }
      }
      guard !allowed else { throw SourceFailure.metadata }
    }
    session.invalidateAndCancel()
    print("SOURCE HTTP REPLAY: real URLSession success/status/declared+stream limits/cancel and redirect delegate PASS")
  }
}
#endif
