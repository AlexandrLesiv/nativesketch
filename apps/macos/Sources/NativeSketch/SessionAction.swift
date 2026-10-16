import CSketchSession

/// Compound user intent. Rust validates before cancelling any active interaction.
enum SessionAction {
  case chooseTool(UInt32)
  enum CycleDirection { case previous, next }
  case cycleSelection(CycleDirection)
  case scaleSelection(Double)
  case deleteSelection, undo, redo

  var value: SketchEditorActionV1 {
    var value = SketchEditorActionV1()
    switch self {
    case .chooseTool(let tool):
      value.tag = UInt32(SK_V1_ACTION_CHOOSE_TOOL)
      value.option = tool
    case .cycleSelection(let direction):
      value.tag = UInt32(SK_V1_ACTION_CYCLE_SELECTION)
      value.option =
        direction == .previous ? UInt32(SK_V1_CYCLE_PREVIOUS) : UInt32(SK_V1_CYCLE_NEXT)
    case .scaleSelection(let factor):
      value.tag = UInt32(SK_V1_ACTION_SCALE_SELECTION)
      value.option = UInt32(SK_V1_ANCHOR_SELECTION_CENTER)
      value.factor = factor
    case .deleteSelection: value.tag = UInt32(SK_V1_ACTION_DELETE)
    case .undo: value.tag = UInt32(SK_V1_ACTION_UNDO)
    case .redo: value.tag = UInt32(SK_V1_ACTION_REDO)
    }
    return value
  }
}

extension RustApplicationSession {
  @discardableResult func perform(_ action: SessionAction) -> Bool {
    guard !unavailable, handle != nil else { return false }
    var value = action.value
    var outcome = SketchOutcomeV1()
    let status = sketch_session_v1_action(runtime.handle, handle, nextContext(), &value, &outcome)
    let accepted = accept(status, outcome)
    runtime.pump.schedule()
    return accepted
  }

  func available(_ action: SessionAction) -> Bool {
    guard !unavailable, handle != nil else { return false }
    var value = action.value
    var result = SketchAvailabilityV1()
    let status = sketch_session_v1_action_availability(runtime.handle, handle, &value, &result)
    return status == SK_V1_OK.rawValue && result.enabled != 0
  }
}
