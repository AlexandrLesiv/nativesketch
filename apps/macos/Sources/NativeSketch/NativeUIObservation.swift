import AppKit
import CSketchSession

/// Observation-only oracle for native UI tests; never changes focus or delivers commands.
@MainActor final class NativeUIObservation {
  private weak var canvas: CanvasView?

  init?(canvas: CanvasView) {
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
    var runtime = SketchRuntimeStateV1()
    let status = sketch_runtime_v1_state(canvas?.owner.runtime.handle, &runtime)
    let storageIdle =
      status == SK_V1_OK.rawValue && runtime.pending_operations == 0
      && runtime.ready_events == 0 && runtime.retained_events == 0
    let projection = canvas?.editor.projection
    return prefix
      + " | Native ready:active=\(NSApp.isActive),key=\(window?.isKeyWindow == true),canvas=\(window?.firstResponder === canvas),field=\(field)"
      + " | Storage idle=\(storageIdle),dirty=\(projection?.dirty == 1),session=\(projection?.session ?? 0)"
      + " | Save outstanding=\(projection?.outstanding_saves ?? 0),preparing=\(projection?.preparing_operations ?? 0)"
      + " | Renderer=Skia,presented=\(canvas?.rendering.presented ?? 0)"
      + " rendererRecoveries=\(canvas?.rendering.recoveries ?? 0),rendererFailures=\(canvas?.rendering.failures ?? 0)"
      + " | Interaction=\(projection?.interaction ?? 0)"
  }
}
