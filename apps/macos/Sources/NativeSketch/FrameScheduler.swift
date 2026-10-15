import AppKit
import Metal

/// Main-thread owner. Repaints coalesce, never input samples.
@MainActor final class FrameScheduler {
  private unowned let canvas: CanvasView
  private let surface: MetalSurface
  private let renderer: SkiaRenderer
  private var state = RenderScheduleState()
  private var requestedAt = 0.0
  private var scheduled = false
  private(set) var recoveries: UInt64 = 0
  private(set) var failures: UInt64 = 0
  private var injectedFailure = false
  private var lastFailure: UInt32?
  private(set) var presented: UInt64 = 0
  init(canvas: CanvasView, surface: MetalSurface) throws {
    self.canvas = canvas
    self.surface = surface
    renderer = try SkiaRenderer(surface: surface)
  }
  func close() {
    state.close()
    renderer.close()
  }
  func invalidate() {
    guard !state.closed else { return }
    if !state.pending { requestedAt = ProcessInfo.processInfo.systemUptime }
    state.request()
    schedule()
  }
  private func schedule() {
    guard state.canSchedule, !scheduled else { return }
    scheduled = true
    DispatchQueue.main.async { [weak self] in
      guard let self else { return }
      self.scheduled = false
      self.render()
    }
  }
  private func render() {
    guard state.canSchedule,
      let window = canvas.window, window.occlusionState.contains(.visible),
      !canvas.isHiddenOrHasHiddenAncestor, canvas.bounds.width > 0, canvas.bounds.height > 0
    else { return }
    if state.recoveryNeeded {
      state.beginRecovery()
      surface.invalidateGeneration()
      do {
        try renderer.reset()
        recoveries += 1
      } catch let error as NativeBridgeError {
        failure(error.status)
        return
      } catch {
        failure(9)
        return
      }
    }
    state.begin()
    surface.resize(canvas.bounds.size, scale: window.backingScaleFactor)
    let acquired = ProcessInfo.processInfo.systemUptime
    // Missing drawables wait for a later invalidation; never busy-retry.
    guard let drawable = surface.layer.nextDrawable(),
      let command = surface.queue.makeCommandBuffer()
    else {
      state.deferDrawable()
      return
    }
    let acquisition = ProcessInfo.processInfo.systemUptime - acquired
    do {
      #if DEBUG && NATIVE_TEST_CONTROLS
        if !injectedFailure && CommandLine.arguments.contains("--test-renderer-failure-once") {
          injectedFailure = true
          throw NativeBridgeError(operation: "test-renderer-failure", status: 9)
        }
      #endif
      let stats = try renderer.draw(canvas: canvas, surface: surface, texture: drawable.texture)
      let session = canvas.editor.projection.session
      let scene = canvas.editor.projection.scene_generation
      let surfaceGeneration = surface.generation
      let frame = canvas.renderDiagnostics.record(
        stats, session: session, scene: scene, surface: surfaceGeneration,
        requestedAt: requestedAt, renderStartedAt: acquired, acquisition: acquisition,
        outstanding: state.outstanding + 1)
      state.submitted()
      drawable.addPresentedHandler { [weak self] drawable in
        let time = drawable.presentedTime
        Task { @MainActor [weak self] in
          guard let self else { return }
          self.canvas.renderDiagnostics.presented(frame, time: time)
          if !self.state.closed && self.surface.generation == surfaceGeneration
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
          self.state.completed()
          self.canvas.renderDiagnostics.completed(frame, failed: failed)
          if self.surface.generation == surfaceGeneration && !self.state.closed {
            if failed { self.failure(9) } else { self.state.succeeded() }
          }
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
    guard !state.closed else { return }
    failures += 1
    state.failed()
    schedule()
    guard lastFailure != status else { return }
    lastFailure = status
    NativeDiagnostics.failure(
      "renderer-frame", status: status, session: canvas.editor.projection.session)
  }
}
