import CSketchFFI
import UIKit
import UniformTypeIdentifiers

extension DrawingController {
  #if !os(visionOS)
    func sendToWatch() {
      canvas.cancelAll()
      guard let (data, _) = editor.snapshot() else { return }
      watchSender.send(data, title: name) { [weak self] message in
        self?.receiveWatchStatus(message)
      }
    }
    func receiveWatchStatus(_ message: String) {
      lastWatchStatus = message
      if presentedViewController == nil { showWatchStatus() }
    }
    func showWatchStatus() {
      guard presentedViewController == nil else { return }
      let alert = UIAlertController(
        title: "Apple Watch Transfer Status", message: lastWatchStatus,
        preferredStyle: .alert)
      alert.addAction(UIAlertAction(title: "OK", style: .default))
      present(alert, animated: true)
    }
  #endif
}
