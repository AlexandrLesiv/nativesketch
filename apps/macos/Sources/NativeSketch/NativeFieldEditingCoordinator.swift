import AppKit

/// Owns only a native field's unfinished edit and rollback text, never editor settings.
@MainActor
final class NativeFieldEditingCoordinator: NSObject, NSTextFieldDelegate,
  NSPopoverDelegate
{
  private unowned let field: NSTextField
  private let report: (String) -> Void
  private var originalText: String?

  init(field: NSTextField, report: @escaping (String) -> Void) {
    self.field = field
    self.report = report
  }

  func controlTextDidBeginEditing(_ notification: Notification) {
    originalText = field.stringValue
  }

  func controlTextDidEndEditing(_ notification: Notification) {
    originalText = nil
  }

  func control(_ control: NSControl, textShouldEndEditing fieldEditor: NSText) -> Bool {
    guard let formatter = field.formatter as? NumberFormatter,
      let number = formatter.number(from: fieldEditor.string), number.doubleValue.isFinite,
      number.doubleValue >= (formatter.minimum?.doubleValue ?? -.infinity),
      number.doubleValue <= (formatter.maximum?.doubleValue ?? .infinity)
    else {
      reject()
      return false
    }
    return true
  }

  func control(
    _ control: NSControl, didFailToFormatString string: String, errorDescription error: String?
  ) -> Bool {
    reject()
    return false
  }

  func control(
    _ control: NSControl, textView: NSTextView, doCommandBy commandSelector: Selector
  ) -> Bool {
    guard commandSelector == #selector(NSResponder.cancelOperation(_:)),
      let originalText
    else { return false }
    textView.string = originalText
    field.stringValue = originalText
    return true
  }

  func finishEditing() -> Bool {
    guard field.currentEditor() != nil else { return true }
    return field.window?.makeFirstResponder(nil) == true
  }

  func popoverShouldClose(_ popover: NSPopover) -> Bool { finishEditing() }

  private func reject() {
    let message = "Enter a stroke width from 0.1 to 256 points, or press Escape to restore it."
    report(message)
    NSSound.beep()
    NSAccessibility.post(
      element: field, notification: .announcementRequested,
      userInfo: [.announcement: message, .priority: NSAccessibilityPriorityLevel.medium.rawValue])
  }
}
