import CSketchSession

struct RenderAcquisitionFailure {
  let startedAt: Double
  let endedAt: Double
  let reason: String
}

struct RenderFrameRecord {
  let id: UInt64
  let session: UInt64
  let sceneGeneration: UInt64
  let surfaceGeneration: UInt64
  let renderStartedAt: Double
  let submittedAt: Double
  let requestedAt: Double
  let inputAt: Double?
  let acquisitionSeconds: Double
  let outstanding: Int
  let stats: SketchRenderStatsV1
  var completedAt: Double?
  var presentedAt: Double?
  var reportedPresentationTime: Double?
  var presentationStatus = "awaiting-callback"
  var failed = false

  mutating func observePresentation(_ observation: RenderPresentationTime) {
    reportedPresentationTime = observation.reportedTime
    presentationStatus = observation.status
    presentedAt = observation.acceptedTime
  }
}
