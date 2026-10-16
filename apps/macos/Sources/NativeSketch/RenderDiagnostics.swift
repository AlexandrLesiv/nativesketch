import CSketchSession
import Foundation

/// Bounded release observations for explicit benchmark export; no per-frame logging.
@MainActor final class RenderDiagnostics {
  private var records: [RenderFrameRecord] = []
  private var acquisitionFailures: [RenderAcquisitionFailure] = []
  private(set) var acquisitionFailureCount: UInt64 = 0
  private var nextID: UInt64 = 0
  private var lastInputUptime: Double?
  // NSEvent.timestamp and systemUptime are seconds since system startup.
  func input(timestamp: Double) { lastInputUptime = timestamp }
  func snapshot() -> [RenderFrameRecord] { records }
  func failedAcquisitions() -> [RenderAcquisitionFailure] { acquisitionFailures }
  func acquisitionFailed(startedAt: Double, reason: String) {
    acquisitionFailureCount += 1
    if acquisitionFailures.count == 256 { acquisitionFailures.removeFirst() }
    acquisitionFailures.append(
      RenderAcquisitionFailure(
        startedAt: startedAt, endedAt: ProcessInfo.processInfo.systemUptime, reason: reason))
  }
  func reset(clearInput: Bool = true) {
    records.removeAll(keepingCapacity: true)
    acquisitionFailures.removeAll(keepingCapacity: true)
    acquisitionFailureCount = 0
    if clearInput { lastInputUptime = nil }
  }

  func record(
    _ stats: SketchRenderStatsV1, session: UInt64, scene: UInt64, surface: UInt64,
    requestedAt: Double, submittedAt: Double, renderStartedAt: Double,
    acquisition: Double, outstanding: Int
  ) -> UInt64 {
    nextID += 1
    if records.count == 256 { records.removeFirst() }
    records.append(
      RenderFrameRecord(
        id: nextID, session: session, sceneGeneration: scene, surfaceGeneration: surface,
        renderStartedAt: renderStartedAt, submittedAt: submittedAt,
        requestedAt: requestedAt, inputAt: lastInputUptime, acquisitionSeconds: acquisition,
        outstanding: outstanding, stats: stats))
    return nextID
  }

  func completed(_ id: UInt64, failed: Bool) {
    guard let index = records.firstIndex(where: { $0.id == id }) else { return }
    records[index].completedAt = ProcessInfo.processInfo.systemUptime
    records[index].failed = failed
  }

  // Metal reports actual presentation host time, not physical pen-to-screen latency.
  func presented(_ id: UInt64, observation: RenderPresentationTime) {
    guard let index = records.firstIndex(where: { $0.id == id }) else { return }
    records[index].observePresentation(observation)
  }
}
