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
    let heading = NSTextField(labelWithString: title)
    heading.font = .boldSystemFont(ofSize: NSFont.systemFontSize)
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
    explanation.preferredMaxLayoutWidth = 400
    cancelButton.target = self
    cancelButton.action = #selector(cancelOperation(_:))
    cancelButton.keyEquivalent = "\r"
    cancelButton.setAccessibilityIdentifier("sketch.navigation.cancel")
    defaultButtonCell = cancelButton.cell as? NSButtonCell
    initialFirstResponder = cancelButton
    guard let contentView else { return }
    for view in [heading, explanation, cancelButton] {
      view.translatesAutoresizingMaskIntoConstraints = false
      contentView.addSubview(view)
    }
    NSLayoutConstraint.activate([
      heading.leadingAnchor.constraint(equalTo: contentView.leadingAnchor, constant: 20),
      heading.topAnchor.constraint(equalTo: contentView.topAnchor, constant: 20),
      explanation.leadingAnchor.constraint(equalTo: contentView.leadingAnchor, constant: 20),
      explanation.trailingAnchor.constraint(equalTo: contentView.trailingAnchor, constant: -20),
      explanation.topAnchor.constraint(equalTo: heading.bottomAnchor, constant: 10),
      cancelButton.topAnchor.constraint(
        greaterThanOrEqualTo: explanation.bottomAnchor, constant: 16),
      cancelButton.trailingAnchor.constraint(equalTo: contentView.trailingAnchor, constant: -20),
      cancelButton.bottomAnchor.constraint(equalTo: contentView.bottomAnchor, constant: -16),
      cancelButton.widthAnchor.constraint(greaterThanOrEqualToConstant: 90),
    ])
    let height =
      62 + heading.fittingSize.height + explanation.fittingSize.height
      + cancelButton.fittingSize.height
    setContentSize(NSSize(width: 440, height: max(180, height)))
  }
  override func cancelOperation(_ sender: Any?) {
    sheetParent?.endSheet(self, returnCode: .cancel)
  }
}
