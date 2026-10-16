import AppKit
import CSketchSession

/// Failure copy distinguishes a confirmed replacement from an indeterminate write.
@MainActor enum NativeFailurePresentation {
  static func alert(_ outcome: SketchOutcomeV1, additionalFailures: Int) -> NSAlert {
    let alert = NSAlert()
    let written = outcome.durability == SK_V1_DIRECTORY_SYNC_UNCONFIRMED.rawValue
    let uncertain =
      outcome.save_result == SK_V1_UNCERTAIN.rawValue
      || outcome.save_result == SK_V1_UNCERTAIN_AFTER_CANCEL.rawValue
    alert.messageText =
      written
      ? "The file was written, but durability is unconfirmed"
      : uncertain ? "The save could not be confirmed" : "The operation could not be completed"
    alert.informativeText =
      written
      ? "Keep this document open and save again when storage is available."
      : uncertain
        ? "The destination may have changed. Your current document is retained; check the file before retrying."
        : "Your current document is retained. Check the destination and available storage."
    if additionalFailures > 0 {
      alert.informativeText += " \(additionalFailures) other operations also reported a failure."
    }
    if SessionCommand.retry(outcome.retry_action) != nil {
      alert.addButton(withTitle: "Try Again").setAccessibilityIdentifier("sketch.failure.retry")
    }
    alert.addButton(withTitle: "Save As…").setAccessibilityIdentifier("sketch.failure.save-as")
    alert.addButton(withTitle: "Cancel").setAccessibilityIdentifier("sketch.failure.cancel")
    return alert
  }
}

/// Bounded presentation state, not pending I/O or domain state. Every failure is logged;
/// excess failures are summarized visibly rather than silently dropped behind a sheet.
struct NativePendingFailure {
  var outcome: SketchOutcomeV1
  var additionalFailures = 0
  func isCurrent(_ projection: SketchProjectionV1) -> Bool {
    (outcome.failure.session == 0 || outcome.failure.session == projection.session)
      && outcome.projection.binding_generation == projection.binding_generation
      && outcome.acknowledgement != SK_V1_STALE_SESSION.rawValue
      && outcome.acknowledgement != SK_V1_STALE_BINDING.rawValue
  }
}
