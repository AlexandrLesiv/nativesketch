import Foundation

struct RenderBenchmarkReport: Encodable {
  var schema = 1
  var valid = false
  var reason: String?
  var metadata: [String: String] = [:]
  var phases: [RenderBenchmarkPhase] = []
  func write(to output: URL) throws {
    let encoder = JSONEncoder()
    encoder.outputFormatting = [.prettyPrinted, .sortedKeys]
    // Preserve earlier attempts, including failures, even when a caller reuses a path.
    try encoder.encode(self).write(to: output, options: .withoutOverwriting)
  }
}

struct RenderBenchmarkPhase: Encodable {
  let name: String
  let startedAt: Double
  let endedAt: Double
  let wallSeconds: Double
  let requestedSteps: Int
  let missingCompletions: Int
  let missingPresentations: Int
  let coalescedOrMissingFrames: Int
  let latestStatePresented: Bool
  let failures: UInt64
  let acquisitionFailureCount: UInt64
  let acquisitionFailureRecordsDropped: UInt64
  let acquisitionAttempts: [RenderBenchmarkAcquisition]
  let records: [RenderBenchmarkFrame]
  init(
    name: String, steps: Int, startedAt: Double, endedAt: Double, failures: UInt64,
    latestStatePresented: Bool,
    acquisitionFailureCount: UInt64, acquisitionFailures: [RenderAcquisitionFailure],
    records: [RenderFrameRecord]
  ) {
    self.name = name
    self.startedAt = startedAt
    self.endedAt = endedAt
    wallSeconds = endedAt - startedAt
    requestedSteps = steps
    self.failures = failures
    self.latestStatePresented = latestStatePresented
    self.acquisitionFailureCount = acquisitionFailureCount
    acquisitionFailureRecordsDropped = acquisitionFailureCount - UInt64(acquisitionFailures.count)
    acquisitionAttempts =
      (records.map {
        RenderBenchmarkAcquisition(
          startedAt: $0.renderStartedAt,
          endedAt: $0.renderStartedAt + $0.acquisitionSeconds, failure: nil)
      }
      + acquisitionFailures.map {
        RenderBenchmarkAcquisition(
          startedAt: $0.startedAt, endedAt: $0.endedAt, failure: $0.reason)
      }).sorted { $0.startedAt < $1.startedAt }
    missingCompletions = records.filter { $0.completedAt == nil }.count
    missingPresentations = records.filter { $0.presentedAt == nil }.count
    coalescedOrMissingFrames = max(0, steps - records.count)
    self.records = records.map(RenderBenchmarkFrame.init)
  }
}

struct RenderBenchmarkFrame: Encodable {
  let frameID: UInt64
  let session: UInt64
  let sceneGeneration: UInt64
  let contextGeneration: UInt64
  let surfaceGeneration: UInt64
  let requestedAt: Double
  let renderStartedAt: Double
  let submittedAt: Double
  let inputAt: Double?
  let completedAt: Double?
  let presentedAt: Double?
  let reportedPresentationTime: Double?
  let presentationStatus: String
  let queueSeconds: Double
  let sampleAgeSeconds: Double?
  let requestToPresentationSeconds: Double?
  let sampleToPresentationSeconds: Double?
  let acquisitionSeconds: Double
  let scenePrepareNS: UInt64
  let renderSubmitNS: UInt64
  let geometryBytes: UInt64
  let pathBytes: UInt64
  let backgroundBytes: UInt64
  let pathHits: UInt64
  let pathMisses: UInt64
  let backgroundHit: Bool
  let outstanding: Int
  let failed: Bool
  init(_ frame: RenderFrameRecord) {
    frameID = frame.id
    session = frame.session
    sceneGeneration = frame.sceneGeneration
    contextGeneration = frame.stats.context_generation
    surfaceGeneration = frame.surfaceGeneration
    requestedAt = frame.requestedAt
    renderStartedAt = frame.renderStartedAt
    submittedAt = frame.submittedAt
    inputAt = frame.inputAt
    completedAt = frame.completedAt
    presentedAt = frame.presentedAt
    reportedPresentationTime = frame.reportedPresentationTime
    presentationStatus = frame.presentationStatus
    queueSeconds = frame.renderStartedAt - frame.requestedAt
    sampleAgeSeconds = frame.inputAt.map { frame.renderStartedAt - $0 }
    requestToPresentationSeconds = frame.presentedAt.map { $0 - frame.requestedAt }
    sampleToPresentationSeconds = frame.inputAt.flatMap { input in
      frame.presentedAt.map { $0 - input }
    }
    acquisitionSeconds = frame.acquisitionSeconds
    scenePrepareNS = frame.stats.scene_prepare_ns
    renderSubmitNS = frame.stats.render_submit_ns
    geometryBytes = frame.stats.geometry_bytes
    pathBytes = frame.stats.path_bytes
    backgroundBytes = frame.stats.background_bytes
    pathHits = frame.stats.path_hits
    pathMisses = frame.stats.path_misses
    backgroundHit = frame.stats.background_hit != 0
    outstanding = frame.outstanding
    failed = frame.failed
  }
}

struct RenderBenchmarkAcquisition: Encodable {
  let startedAt: Double
  let endedAt: Double
  let seconds: Double
  let failure: String?
  init(startedAt: Double, endedAt: Double, failure: String?) {
    self.startedAt = startedAt
    self.endedAt = endedAt
    seconds = endedAt - startedAt
    self.failure = failure
  }
}
