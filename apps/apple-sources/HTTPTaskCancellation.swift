import CryptoKit
import Foundation

final class HTTPTaskCancellation: @unchecked Sendable {
  private let lock = NSLock()
  private var task: URLSessionTask?
  private var cancelled = false
  func install(_ task: URLSessionTask) {
    lock.lock()
    self.task = task
    let stop = cancelled
    lock.unlock()
    if stop { task.cancel() }
  }
  func cancel() {
    lock.lock()
    cancelled = true
    let current = task
    lock.unlock()
    current?.cancel()
  }
}
