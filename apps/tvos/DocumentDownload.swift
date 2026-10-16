import Darwin
import Foundation
import UIKit

// Delegate streaming bounds memory before appending; redirects cannot downgrade HTTPS.
final class DocumentDownload: NSObject, URLSessionDataDelegate, @unchecked Sendable {
  static let limit = 64 * 1024 * 1024
  private var body = BoundedDownload(limit: DocumentDownload.limit)
  private let sessionLock = NSLock()
  private var session: URLSession?
  private let completion: @Sendable (Result<Data, Error>) -> Void
  init(url: URL, completion: @escaping @Sendable (Result<Data, Error>) -> Void) {
    self.completion = completion
    super.init()
    let configuration = URLSessionConfiguration.ephemeral
    configuration.timeoutIntervalForResource = 120
    let queue = OperationQueue()
    queue.maxConcurrentOperationCount = 1
    let created = URLSession(configuration: configuration, delegate: self, delegateQueue: queue)
    sessionLock.withLock { session = created }
    created.dataTask(with: url).resume()
  }
  func cancel() {
    let current = sessionLock.withLock { session }
    current?.invalidateAndCancel()
  }
  func urlSession(
    _ session: URLSession, task: URLSessionTask,
    willPerformHTTPRedirection response: HTTPURLResponse, newRequest request: URLRequest,
    completionHandler: @escaping @Sendable (URLRequest?) -> Void
  ) {
    completionHandler(body.redirect(to: request.url) ? request : nil)
  }
  func urlSession(
    _ session: URLSession, dataTask: URLSessionDataTask, didReceive response: URLResponse,
    completionHandler: @escaping @Sendable (URLSession.ResponseDisposition) -> Void
  ) {
    guard
      body.response(
        status: (response as? HTTPURLResponse)?.statusCode,
        expectedLength: response.expectedContentLength)
    else {
      completionHandler(.cancel)
      return
    }
    completionHandler(.allow)
  }
  func urlSession(_ session: URLSession, dataTask: URLSessionDataTask, didReceive data: Data) {
    if !body.append(data) { dataTask.cancel() }
  }
  func urlSession(_ session: URLSession, task: URLSessionTask, didCompleteWithError error: Error?) {
    completion(body.result(transportError: error))
    session.finishTasksAndInvalidate()
    sessionLock.withLock { self.session = nil }
  }
}
