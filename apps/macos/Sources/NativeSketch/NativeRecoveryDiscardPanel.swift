import AppKit

/// Native destructive confirmation with non-destructive Return and Escape routes.
@MainActor final class NativeRecoveryDiscardPanel: NSPanel {
  let cancelButton = NSButton(title: "Cancel", target: nil, action: nil)
  init() {
    super.init(
      contentRect: NSRect(x: 0, y: 0, width: 440, height: 180),
      styleMask: [.titled], backing: .buffered, defer: false)
    title = "Discard this recovery copy?"
    setAccessibilityIdentifier("sketch.recovery.confirmation")
    let heading = NSTextField(labelWithString: title)
    heading.font = .boldSystemFont(ofSize: NSFont.systemFontSize)
    let explanation = NSTextField(
      wrappingLabelWithString: "This removes only the selected recovery generation.")
    explanation.preferredMaxLayoutWidth = 400
    cancelButton.target = self
    cancelButton.action = #selector(cancelOperation(_:))
    cancelButton.keyEquivalent = "\r"
    cancelButton.setAccessibilityIdentifier("sketch.recovery.confirm-cancel")
    let discard = NSButton(title: "Discard", target: self, action: #selector(discardCopy))
    discard.setAccessibilityIdentifier("sketch.recovery.confirm-discard")
    defaultButtonCell = cancelButton.cell as? NSButtonCell
    initialFirstResponder = cancelButton
    let buttons = NSStackView(views: [cancelButton, discard])
    buttons.spacing = 8
    guard let contentView else { return }
    for view in [heading, explanation, buttons] {
      view.translatesAutoresizingMaskIntoConstraints = false
      contentView.addSubview(view)
    }
    NSLayoutConstraint.activate([
      heading.leadingAnchor.constraint(equalTo: contentView.leadingAnchor, constant: 20),
      heading.topAnchor.constraint(equalTo: contentView.topAnchor, constant: 20),
      explanation.leadingAnchor.constraint(equalTo: contentView.leadingAnchor, constant: 20),
      explanation.trailingAnchor.constraint(equalTo: contentView.trailingAnchor, constant: -20),
      explanation.topAnchor.constraint(equalTo: heading.bottomAnchor, constant: 10),
      buttons.topAnchor.constraint(greaterThanOrEqualTo: explanation.bottomAnchor, constant: 16),
      buttons.trailingAnchor.constraint(equalTo: contentView.trailingAnchor, constant: -20),
      buttons.bottomAnchor.constraint(equalTo: contentView.bottomAnchor, constant: -16),
      cancelButton.widthAnchor.constraint(greaterThanOrEqualToConstant: 90),
      discard.widthAnchor.constraint(greaterThanOrEqualToConstant: 90),
    ])
    let height =
      62 + heading.fittingSize.height + explanation.fittingSize.height + buttons.fittingSize.height
    setContentSize(NSSize(width: 440, height: max(180, height)))
  }
  override func cancelOperation(_ sender: Any?) {
    sheetParent?.endSheet(self, returnCode: .cancel)
  }
  @objc private func discardCopy() { sheetParent?.endSheet(self, returnCode: .OK) }
}
