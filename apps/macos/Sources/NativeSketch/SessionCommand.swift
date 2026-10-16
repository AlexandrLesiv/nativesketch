import CSketchSession
import Foundation

/// Native intent values only. Rust validates and executes every domain command.
enum SessionCommand {
  case tool(UInt32)
  case brush(SketchBrushV1)
  case select(Double, Double, Double)
  case cycle(Int32)
  case clearSelection
  case beginStroke(UInt64, UInt32)
  case commitStroke(UInt64)
  case beginTransform
  case transform(Double, Double, Double)
  case commitTransform, cancel
  case delete, undo, redo, save, saveAs, exportCopy, new, open, close
  case recover(UInt64)

  var value: SketchCommandV1 {
    var value = SketchCommandV1()
    func tag(_ tag: SketchCommandTagV1) { value.tag = tag.rawValue }
    switch self {
    case .tool(let tool):
      tag(SK_V1_SET_TOOL)
      value.payload.tool = tool
    case .brush(let brush):
      tag(SK_V1_SET_BRUSH)
      value.payload.brush = brush
    case .select(let x, let y, let tolerance):
      tag(SK_V1_SELECT_AT)
      value.payload.selection.x = x
      value.payload.selection.y = y
      value.payload.selection.tolerance = tolerance
    case .cycle(let direction):
      tag(SK_V1_SELECT_CYCLE)
      value.payload.direction = direction
    case .clearSelection: tag(SK_V1_CLEAR_SELECTION)
    case .beginStroke(let pointer, let device):
      tag(SK_V1_BEGIN_STROKE)
      value.payload.stroke.pointer = pointer
      value.payload.stroke.device = device
    case .commitStroke(let pointer):
      tag(SK_V1_COMMIT_STROKE)
      value.payload.stroke.pointer = pointer
    case .beginTransform: tag(SK_V1_BEGIN_TRANSFORM)
    case .transform(let scale, let x, let y):
      tag(SK_V1_UPDATE_TRANSFORM)
      value.payload.transform.scale = scale
      value.payload.transform.anchor_x = x
      value.payload.transform.anchor_y = y
    case .commitTransform: tag(SK_V1_COMMIT_TRANSFORM)
    case .cancel: tag(SK_V1_CANCEL_INTERACTION)
    case .delete: tag(SK_V1_DELETE_SELECTION)
    case .undo: tag(SK_V1_UNDO)
    case .redo: tag(SK_V1_REDO)
    case .save: tag(SK_V1_SAVE)
    case .saveAs: tag(SK_V1_SAVE_AS)
    case .exportCopy: tag(SK_V1_EXPORT_COPY)
    case .new: tag(SK_V1_NEW)
    case .open: tag(SK_V1_OPEN)
    case .close: tag(SK_V1_CLOSE)
    case .recover(let item):
      tag(SK_V1_OPEN_RECOVERY)
      value.payload.recovery_item = item
    }
    return value
  }

  static func retry(_ tag: UInt32) -> Self? {
    switch tag {
    case SK_V1_SAVE.rawValue: .save
    case SK_V1_SAVE_AS.rawValue: .saveAs
    case SK_V1_EXPORT_COPY.rawValue: .exportCopy
    case SK_V1_OPEN.rawValue: .open
    default: nil
    }
  }
}
