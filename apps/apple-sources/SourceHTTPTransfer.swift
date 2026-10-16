import CryptoKit
import Foundation

// Serial delegate queue exclusively owns response bytes and errors.
final class SourceHTTPTransfer: NSObject, URLSessionDataDelegate, @unchecked Sendable {
  private var bytes = Data()
  private var failure: Error?
  private let limit: Int
  private let origin: URL
  private let completion: @Sendable (Result<Data, Error>) -> Void
  init(origin: URL, limit: Int, completion: @escaping @Sendable (Result<Data, Error>) -> Void) {
    self.origin = origin
    self.limit = limit
    self.completion = completion
  }
  func urlSession(
    _ session: URLSession, task: URLSessionTask,
    willPerformHTTPRedirection response: HTTPURLResponse, newRequest request: URLRequest,
    completionHandler: @escaping @Sendable (URLRequest?) -> Void
  ) {
    guard let url = request.url, url.scheme == "https", url.host == origin.host,
      url.port == origin.port
    else {
      failure = SourceFailure.http
      completionHandler(nil)
      return
    }
    completionHandler(request)
  }
  func urlSession(
    _ session: URLSession, dataTask: URLSessionDataTask, didReceive response: URLResponse,
    completionHandler: @escaping @Sendable (URLSession.ResponseDisposition) -> Void
  ) {
    guard let http = response as? HTTPURLResponse, (200...299).contains(http.statusCode),
      response.expectedContentLength <= limit
    else {
      failure = SourceFailure.http
      completionHandler(.cancel)
      return
    }
    completionHandler(.allow)
  }
  func urlSession(_ session: URLSession, dataTask: URLSessionDataTask, didReceive data: Data) {
    guard data.count <= limit, bytes.count <= limit - data.count else {
      failure = SourceFailure.oversized
      dataTask.cancel()
      return
    }
    bytes.append(data)
  }
  func urlSession(_ session: URLSession, task: URLSessionTask, didCompleteWithError error: Error?) {
    completion((failure ?? error).map { .failure($0) } ?? .success(bytes))
    session.finishTasksAndInvalidate()
  }
  static func get(_ url: URL, limit: Int, configuration: URLSessionConfiguration = .ephemeral)
    async throws -> Data
  {
    let cancellation = HTTPTaskCancellation()
    return try await withTaskCancellationHandler {
      try await withCheckedThrowingContinuation { continuation in
        let delegate = SourceHTTPTransfer(origin: url, limit: limit) {
          continuation.resume(with: $0)
        }
        let config = configuration
        config.timeoutIntervalForRequest = 15
        config.timeoutIntervalForResource = 20
        config.httpCookieStorage = nil
        config.urlCredentialStorage = nil
        config.urlCache = nil
        let queue = OperationQueue()
        queue.maxConcurrentOperationCount = 1
        let task = URLSession(configuration: config, delegate: delegate, delegateQueue: queue)
          .dataTask(with: url)
        cancellation.install(task)
        task.resume()
      }
    } onCancel: {
      cancellation.cancel()
    }
  }

}
