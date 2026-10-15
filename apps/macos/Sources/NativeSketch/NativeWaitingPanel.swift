import AppKit
import CSketchSession

/// A native sheet with Return's default button and responder-chain Escape cancellation.
@MainActor final class NativeWaitingPanel: NSPanel {
  let cancelButton = NSButton(title: "Cancel", target: nil, action: nil)
  init(navigation: UInt32, terminating: Bool) {
    super.init(
      contentRect: NSRect(x: 0, y: 0, width: 440, height: 180),
      styleMask: [.titled], backing: .buffered, defer: false)
    title = "Finishing Save…"
    setAccessibilityIdentifier("sketch.navigation.waiting")
    let destination: String
    switch navigation {
    case SK_V1_NAV_NEW.rawValue: destination = "creating a new drawing"
    case SK_V1_NAV_OPEN.rawValue, SK_V1_NAV_RECOVER.rawValue:
      destination = "opening another drawing"
    default: destination = terminating ? "quitting" : "closing this drawing"
    }
    let explanation = NSTextField(
      wrappingLabelWithString:
        "Waiting for pending saves before \(destination). Cancelling keeps this drawing open; saves already in progress will still finish."
    )
    explanation.setAccessibilityIdentifier("sketch.navigation.explanation")
    cancelButton.target = self
    cancelButton.action = #selector(cancelOperation(_:))
    cancelButton.keyEquivalent = "\r"
    cancelButton.setAccessibilityIdentifier("sketch.navigation.cancel")
    defaultButtonCell = cancelButton.cell as? NSButtonCell
    guard let contentView else { return }
    for view in [explanation, cancelButton] {
      view.translatesAutoresizingMaskIntoConstraints = false
      contentView.addSubview(view)
    }
    NSLayoutConstraint.activate([
      explanation.leadingAnchor.constraint(equalTo: contentView.leadingAnchor, constant: 20),
      explanation.trailingAnchor.constraint(equalTo: contentView.trailingAnchor, constant: -20),
      explanation.topAnchor.constraint(equalTo: contentView.topAnchor, constant: 20),
      cancelButton.trailingAnchor.constraint(equalTo: contentView.trailingAnchor, constant: -20),
      cancelButton.bottomAnchor.constraint(equalTo: contentView.bottomAnchor, constant: -16),
      cancelButton.widthAnchor.constraint(greaterThanOrEqualToConstant: 90),
    ])
  }
  override func cancelOperation(_ sender: Any?) {
    sheetParent?.endSheet(self, returnCode: .cancel)
  }
}
