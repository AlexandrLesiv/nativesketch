import AppKit

/// Observation-only oracle for native UI tests; never changes focus or delivers commands.
@MainActor final class NativeUIObservation {
  private weak var canvas: NSView?

  init?(canvas: NSView) {
    #if DEBUG
      guard CommandLine.arguments.contains("--native-ui-observation") else { return nil }
      self.canvas = canvas
    #else
      return nil
    #endif
  }

  func value(appendingTo base: Any?) -> String {
    let prefix = base as? String ?? ""
    let window = canvas?.window
    let field = (NSApp.keyWindow?.firstResponder as? NSTextView)?.isFieldEditor == true
    return prefix
      + " | Native ready:active=\(NSApp.isActive),key=\(window?.isKeyWindow == true),canvas=\(window?.firstResponder === canvas),field=\(field)"
  }
}
