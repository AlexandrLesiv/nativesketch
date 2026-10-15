import AppKit

/// Explicit foreground benchmark only. Never entered by ordinary launches or headless tests.
@MainActor final class RenderBenchmarkRunner: NSObject, NSApplicationDelegate {
  private let output: URL
  private let owner: EditorWindowController
  private var visibility: RenderBenchmarkVisibility?
  private var report = RenderBenchmarkReport()
  private var timer: Timer?
  private let phases = [
    ("cold", 1), ("warm-static", 120), ("live", 120),
    ("commit", 1), ("pan", 120), ("after-idle", 1),
  ]
  private var phaseIndex = 0
  private var stepIndex = 0
  private var phaseStartedAt = 0.0
  private var phaseFailures: UInt64 = 0
  private var drainUntil: Double?
  private var idleUntil: Double?
  private var finished = false
  private var visibilityChecks = 0
  private var visibilitySeconds = 0.0
  private var visibilityMaxSeconds = 0.0

  static func run(output: URL) throws {
    let app = NSApplication.shared
    app.setActivationPolicy(.regular)
    let runner: RenderBenchmarkRunner
    do { runner = try RenderBenchmarkRunner(output: output) } catch {
      var failure = RenderBenchmarkReport()
      failure.reason = "benchmark-initialization-failed"
      try failure.write(to: output)
      throw error
    }
    app.delegate = runner
    withExtendedLifetime(runner) { app.run() }
  }

  private init(output: URL) throws {
    self.output = output
    owner = try EditorWindowController(runtime: NativeRuntime(services: false))
    super.init()
  }

  func applicationDidFinishLaunching(_ notification: Notification) {
    do {
      if let fixture = NativeLaunchConfiguration.argument("--benchmark-fixture"),
        fixture != RenderBenchmarkFixture.identity
      {
        throw RenderBenchmarkError(reason: "unsupported-benchmark-fixture")
      }
      try RenderBenchmarkFixture.prepare(owner.editor)
      owner.show()
      owner.window.contentView?.layoutSubtreeIfNeeded()
      phaseStartedAt = ProcessInfo.processInfo.systemUptime
      owner.window.title = "Native Sketch — Renderer software comparison"
      owner.canvas.renderDiagnostics.reset()
      visibility = try RenderBenchmarkVisibility(owner: owner)
      try visibility?.validate(owner)
      report.metadata = visibility!.metadata(owner: owner)
      report.metadata["sourceRevision"] =
        NativeLaunchConfiguration.argument("--benchmark-revision") ?? "unrecorded"
      report.metadata["thermalStateStart"] = String(ProcessInfo.processInfo.thermalState.rawValue)
      report.metadata["lowPowerMode"] = String(ProcessInfo.processInfo.isLowPowerModeEnabled)
      report.metadata["cadence"] =
        "60 requested steps/second, 0.5 second phase drains, 1 second idle"
      // Initial validation precedes the queued first frame; subsequent turns stay sampled.
      timer = Timer.scheduledTimer(withTimeInterval: 1.0 / 60, repeats: true) { [weak self] _ in
        MainActor.assumeIsolated { self?.tick() }
      }
    } catch { finish(error) }
  }

  private func tick() {
    guard !finished else { return }
    do {
      let validationStart = ProcessInfo.processInfo.systemUptime
      try visibility?.validate(owner)
      let validationDuration = ProcessInfo.processInfo.systemUptime - validationStart
      visibilityChecks += 1
      visibilitySeconds += validationDuration
      visibilityMaxSeconds = max(visibilityMaxSeconds, validationDuration)
      let now = ProcessInfo.processInfo.systemUptime
      if let idleUntil {
        guard now >= idleUntil else { return }
        self.idleUntil = nil
        phaseStartedAt = now
        owner.canvas.renderDiagnostics.reset()
      }
      if let drainUntil {
        guard now >= drainUntil else { return }
        appendPhase()
        phaseIndex += 1
        if phaseIndex == phases.count {
          finish(nil)
          return
        }
        stepIndex = 0
        self.drainUntil = nil
        phaseStartedAt = now
        phaseFailures = owner.canvas.rendering.failures
        owner.canvas.renderDiagnostics.reset()
        if phases[phaseIndex].0 == "after-idle" {
          idleUntil = now + 1
          return
        }
      }
      let phase = phases[phaseIndex]
      // show() already queued the cold first frame; do not warm it with another invalidation.
      if phase.0 != "cold" {
        try RenderBenchmarkFixture.step(phase.0, index: stepIndex, owner: owner)
      }
      stepIndex += 1
      if stepIndex == phase.1 { self.drainUntil = now + 0.5 }
    } catch { finish(error) }
  }

  private func appendPhase() {
    let phase = phases[phaseIndex]
    report.phases.append(
      RenderBenchmarkPhase(
        name: phase.0, steps: stepIndex, startedAt: phaseStartedAt,
        endedAt: ProcessInfo.processInfo.systemUptime,
        failures: owner.canvas.rendering.failures - phaseFailures,
        acquisitionFailureCount: owner.canvas.renderDiagnostics.acquisitionFailureCount,
        acquisitionFailures: owner.canvas.renderDiagnostics.failedAcquisitions(),
        records: owner.canvas.renderDiagnostics.snapshot()))
  }

  private func finish(_ error: Error?) {
    guard !finished else { return }
    finished = true
    timer?.invalidate()
    timer = nil
    if error != nil && owner.canvas != nil && phaseIndex < phases.count { appendPhase() }
    report.valid = error == nil
    if error == nil
      && report.phases.contains(where: {
        $0.failures > 0 || $0.missingCompletions > 0 || $0.missingPresentations > 0
          || $0.records.isEmpty
      })
    {
      report.valid = false
      report.reason = "renderer-failure-or-incomplete-presentation"
    }
    if let failure = error as? RenderBenchmarkError { report.reason = failure.reason }
    if error != nil && report.reason == nil { report.reason = "benchmark-operation-failed" }
    report.metadata["visibilityChecks"] = String(visibilityChecks)
    report.metadata["visibilityTotalSeconds"] = String(visibilitySeconds)
    report.metadata["visibilityMaxSeconds"] = String(visibilityMaxSeconds)
    report.metadata["measurementScope"] =
      "software replay through production cached renderer plus 60Hz visibility validation overhead; no physical latency claim"
    report.metadata["thermalStateEnd"] = String(ProcessInfo.processInfo.thermalState.rawValue)
    do {
      try report.write(to: output)
      print(
        "Renderer software benchmark artifact written; valid=\(report.valid). Physical latency not measured."
      )
    } catch {
      fputs("Could not write renderer benchmark artifact.\n", stderr)
      exit(1)
    }
    owner.canvas?.rendering.close()
    // Dedicated disposable process only: bypass application dirty-document close dialogs.
    exit(report.valid ? 0 : 2)
  }
}
