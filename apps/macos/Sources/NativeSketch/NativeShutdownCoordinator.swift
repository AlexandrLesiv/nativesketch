import CSketchSession

/// Owns exactly one platform termination continuation, independently of storage state.
@MainActor final class NativeShutdownCoordinator {
  private var completion: (() -> Void)?
  private var failure: (() -> Void)?
  private var admitted = false
  var awaitingRetirement: Bool { completion != nil && !admitted }

  func request(completion: @escaping () -> Void, failure: (() -> Void)?) {
    guard self.completion == nil else { return }
    self.completion = completion
    self.failure = failure
  }
  func advance(retired: Bool, admit: () -> UInt32) {
    guard awaitingRetirement, retired else { return }
    let status = admit()
    if status == SK_V1_OK.rawValue {
      admitted = true
    } else {
      NativeDiagnostics.failure("shutdown", status: status)
      let callback = failure
      completion = nil
      failure = nil
      callback?()
    }
  }
  func finish() {
    let callback = completion
    completion = nil
    failure = nil
    callback?()
  }
}
