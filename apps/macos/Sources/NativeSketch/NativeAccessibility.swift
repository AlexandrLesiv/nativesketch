import AppKit

/// Stable automation identity supplements, and never replaces, the spoken native label.
@MainActor
enum NativeAccessibility {
  static func label(_ view: NSView, id: String, text: String) {
    view.setAccessibilityIdentifier(id)
    view.setAccessibilityLabel(text)
  }
}
