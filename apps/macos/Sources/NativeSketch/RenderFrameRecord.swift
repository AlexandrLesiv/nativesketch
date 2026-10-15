import CSketchSession

struct RenderFrameRecord {
  let id: UInt64
  let session: UInt64
  let sceneGeneration: UInt64
  let surfaceGeneration: UInt64
  let requestedAt: Double
  let inputAt: Double?
  let acquisitionSeconds: Double
  let outstanding: Int
  let stats: SketchRenderStatsV1
  var completedAt: Double?
  var presentedAt: Double?
  var failed = false
}
