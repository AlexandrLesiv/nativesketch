import CSketchSession
import Foundation

/// Bounded release observations for explicit benchmark export; no per-frame logging.
@MainActor final class RenderDiagnostics {
  private var records: [RenderFrameRecord] = []
  private var nextID: UInt64 = 0
  private var lastInputUptime: Double?
  // NSEvent.timestamp and systemUptime are seconds since system startup.
  func input(timestamp: Double) { lastInputUptime = timestamp }
  func snapshot() -> [RenderFrameRecord] { records }
  func reset() { records.removeAll(keepingCapacity: true) }

  func record(
    _ stats: SketchRenderStatsV1, session: UInt64, scene: UInt64, surface: UInt64,
    requestedAt: Double, acquisition: Double, outstanding: Int
  ) -> UInt64 {
    nextID += 1
    if records.count == 256 { records.removeFirst() }
    records.append(
      RenderFrameRecord(
        id: nextID, session: session, sceneGeneration: scene, surfaceGeneration: surface,
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
  func presented(_ id: UInt64, time: Double) {
    guard let index = records.firstIndex(where: { $0.id == id }) else { return }
    records[index].presentedAt = time
  }
}
