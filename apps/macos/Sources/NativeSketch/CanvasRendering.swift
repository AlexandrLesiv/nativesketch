import AppKit

/// Native surface lifecycle is independent of document/session lifetime.
@MainActor final class CanvasRendering {
  private unowned let canvas: CanvasView
  private var surface: MetalSurface?
  private var scheduler: FrameScheduler?
  private var observers: [NSObjectProtocol] = []
  private var initializationAttempts = 0
  var recoveries: UInt64 { scheduler?.recoveries ?? 0 }
  var failures: UInt64 { scheduler?.failures ?? 0 }
  var presented: UInt64 { scheduler?.presented ?? 0 }
  var currentScenePresented: Bool { scheduler?.currentScenePresented ?? false }
  init(canvas: CanvasView) { self.canvas = canvas }

  func attach() {
    for observer in observers { NotificationCenter.default.removeObserver(observer) }
    observers.removeAll()
    guard let window = canvas.window else {
      scheduler?.close()
      scheduler = nil
      surface = nil
      return
    }
    for name in [
      NSWindow.didChangeOcclusionStateNotification, NSWindow.didChangeScreenNotification,
    ] {
      observers.append(
        NotificationCenter.default.addObserver(forName: name, object: window, queue: .main) {
          [weak self] _ in
          MainActor.assumeIsolated {
            guard let self else { return }
            if self.surface == nil { self.attach() } else { self.invalidate() }
          }
        })
    }
    observers.append(
      NotificationCenter.default.addObserver(
        forName: NSWindow.willCloseNotification, object: window, queue: .main
      ) { [weak self] _ in
        MainActor.assumeIsolated { self?.close() }
      })
    if surface == nil && initializationAttempts < 2 {
      initializationAttempts += 1
      do {
        let surface = try MetalSurface()
        scheduler = try FrameScheduler(canvas: canvas, surface: surface)
        self.surface = surface
        canvas.layer = surface.layer
        canvas.wantsLayer = true
      } catch let error as NativeBridgeError {
        NativeDiagnostics.failure(error.operation, status: error.status)
      } catch {
        NativeDiagnostics.failure("renderer-initialize", status: 9)
      }
    }
    invalidate()
  }
  func close() {
    for observer in observers { NotificationCenter.default.removeObserver(observer) }
    observers.removeAll()
    scheduler?.close()
    scheduler = nil
    surface = nil
  }
  func invalidate() { scheduler?.invalidate() }
  func setSuspended(_ value: Bool) { scheduler?.setSuspended(value) }
}
