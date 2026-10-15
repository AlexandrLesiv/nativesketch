import AppKit
import Metal

/// Main-thread owner. Repaints coalesce, never input samples.
@MainActor final class FrameScheduler {
  private unowned let canvas: CanvasView
  private let surface: MetalSurface
  private let renderer: SkiaRenderer
  private var pending = false
  private var requestedAt = 0.0
  private var scheduled = false
  private var outstanding = 0
  private var closed = false
  private var lastFailure: UInt32?
  private(set) var presented: UInt64 = 0
  init(canvas: CanvasView, surface: MetalSurface) throws {
    self.canvas = canvas
    self.surface = surface
    renderer = try SkiaRenderer(surface: surface)
  }
  func close() {
    closed = true
    pending = false
    renderer.close()
  }
  func invalidate() {
    guard !closed else { return }
    if !pending { requestedAt = ProcessInfo.processInfo.systemUptime }
    pending = true
    schedule()
  }
  private func schedule() {
    guard pending, !scheduled, outstanding < 2, !closed else { return }
    scheduled = true
    DispatchQueue.main.async { [weak self] in
      guard let self else { return }
      self.scheduled = false
      self.render()
    }
  }
  private func render() {
    guard !closed, pending, outstanding < 2,
      let window = canvas.window, window.occlusionState.contains(.visible),
      !canvas.isHiddenOrHasHiddenAncestor, canvas.bounds.width > 0, canvas.bounds.height > 0
    else { return }
    pending = false
    surface.resize(canvas.bounds.size, scale: window.backingScaleFactor)
    let acquired = ProcessInfo.processInfo.systemUptime
    // Missing drawables wait for a later invalidation; never busy-retry.
    guard let drawable = surface.layer.nextDrawable(),
      let command = surface.queue.makeCommandBuffer()
    else { return }
    let acquisition = ProcessInfo.processInfo.systemUptime - acquired
    do {
      let stats = try renderer.draw(canvas: canvas, surface: surface, texture: drawable.texture)
      let session = canvas.editor.projection.session
      let scene = canvas.editor.projection.scene_generation
      let surfaceGeneration = surface.generation
      let frame = canvas.renderDiagnostics.record(
        stats, session: session, scene: scene, surface: surfaceGeneration,
        requestedAt: requestedAt, acquisition: acquisition, outstanding: outstanding + 1)
      outstanding += 1
      drawable.addPresentedHandler { [weak self] drawable in
        let time = drawable.presentedTime
        Task { @MainActor [weak self] in
          guard let self else { return }
          self.canvas.renderDiagnostics.presented(frame, time: time)
          if !self.closed && self.surface.generation == surfaceGeneration
            && self.canvas.editor.projection.session == session
            && self.canvas.editor.projection.scene_generation == scene
          {
            self.presented += 1
          }
        }
      }
      command.present(drawable)
      command.addCompletedHandler { [weak self] buffer in
        let failed = buffer.status == .error
        Task { @MainActor [weak self] in
          guard let self else { return }
          self.outstanding -= 1
          self.canvas.renderDiagnostics.completed(frame, failed: failed)
          if failed { self.failure(9) }
          self.schedule()
        }
      }
      command.commit()
      lastFailure = nil
    } catch let error as NativeBridgeError {
      failure(error.status)
    } catch {
      failure(9)
    }
  }
  private func failure(_ status: UInt32) {
    guard lastFailure != status else { return }
    lastFailure = status
    NativeDiagnostics.failure(
      "renderer-frame", status: status, session: canvas.editor.projection.session)
  }
}
