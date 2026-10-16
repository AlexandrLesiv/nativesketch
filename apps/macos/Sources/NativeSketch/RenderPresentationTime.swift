import Foundation

struct RenderPresentationTime {
  let reportedTime: Double?
  let status: String
  let acceptedTime: Double?
  init(reported: Double, submittedAt: Double) {
    reportedTime = reported.isFinite ? reported : nil
    if reported == 0 {
      status = "not-presented-or-dropped"
      acceptedTime = nil
    } else if !reported.isFinite || reported < submittedAt || reported < 0 {
      status = "invalid-timestamp"
      acceptedTime = nil
    } else {
      status = "presented"
      acceptedTime = reported
    }
  }
}
