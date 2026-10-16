#ifndef SKETCH_COMMANDS_V1_H
#define SKETCH_COMMANDS_V1_H
#include "sketch_values_v1.h"
enum SketchCommandTagV1 {
  SK_V1_SET_TOOL = 1, SK_V1_SET_BRUSH = 2, SK_V1_SELECT_AT = 3,
  SK_V1_SELECT_CYCLE = 4, SK_V1_CLEAR_SELECTION = 5, SK_V1_BEGIN_STROKE = 6,
  SK_V1_COMMIT_STROKE = 7, SK_V1_BEGIN_TRANSFORM = 8, SK_V1_UPDATE_TRANSFORM = 9,
  SK_V1_COMMIT_TRANSFORM = 10, SK_V1_CANCEL_INTERACTION = 11,
  SK_V1_DELETE_SELECTION = 12, SK_V1_UNDO = 13, SK_V1_REDO = 14,
  SK_V1_SAVE = 20, SK_V1_SAVE_AS = 21, SK_V1_EXPORT_COPY = 22,
  SK_V1_NEW = 23, SK_V1_OPEN = 24, SK_V1_CLOSE = 25, SK_V1_OPEN_RECOVERY = 26
};
typedef union {
  uint32_t tool; /* pen1, select2 */
  SketchBrushV1 brush;
  struct { double x, y, tolerance; } selection;
  int32_t direction;
  /* Begin uses pointer/device; Commit uses pointer and requires device/reserved zero. */
  struct { uint64_t pointer; uint32_t device, reserved; } stroke;
  struct { double scale, anchor_x, anchor_y; } transform;
  uint64_t recovery_item; /* runtime-issued inventory item */
} SketchCommandPayloadV1;
typedef struct {
  uint32_t tag, reserved;
  SketchCommandPayloadV1 payload;
} SketchCommandV1;
enum SketchReplyTagV1 {
  SK_V1_REPLY_CANCEL = 1, SK_V1_REPLY_SAVE = 2, SK_V1_REPLY_DISCARD = 3,
  SK_V1_REPLY_CONTINUE = 4, SK_V1_REPLY_CONFIRM_CLOSE = 5
};
enum SketchEffectKindV1 {
  SK_V1_EFFECT_NONE = 0, SK_V1_CHOOSE_DESTINATION = 1, SK_V1_CHOOSE_SOURCE = 2,
  SK_V1_CONFIRM_UNSAVED = 3, SK_V1_AWAIT_WRITES = 4, SK_V1_SAVE_QUEUED = 5,
  SK_V1_PREPARING = 6, SK_V1_OPENING = 7, SK_V1_CLOSE_READY = 8,
  SK_V1_CLOSE_CONFIRMED = 9, SK_V1_CANCELLED = 10, SK_V1_FAILURE = 11
};
enum SketchSaveResultV1 {
  SK_V1_SAVE_NONE = 0, SK_V1_WRITTEN = 1, SK_V1_UNCERTAIN = 2, SK_V1_SAVE_FAILED = 3,
  SK_V1_CANCELLED_BEFORE_START = 4, SK_V1_WRITTEN_AFTER_CANCEL = 5,
  SK_V1_FAILED_AFTER_CANCEL = 6, SK_V1_UNCERTAIN_AFTER_CANCEL = 7,
  SK_V1_DISPATCHED_UNCONFIRMED = 8, SK_V1_DISPATCHED_AFTER_CANCEL = 9
};
enum SketchAcknowledgementV1 {
  SK_V1_ACK_NONE = 0, SK_V1_SAVED_CURRENT = 1, SK_V1_SAVED_OLDER = 2,
  SK_V1_COPY_WRITTEN = 3, SK_V1_CHECKPOINT_WRITTEN = 4, SK_V1_STALE_SESSION = 5,
  SK_V1_STALE_BINDING = 6, SK_V1_SUPERSEDED = 7, SK_V1_ACK_CANCELLED = 8,
  SK_V1_ACK_FAILED = 9, SK_V1_COPY_DISPATCHED = 10
};
enum SketchDurabilityV1 {
  SK_V1_DURABILITY_NONE = 0, SK_V1_FILE_DIRECTORY_SYNCED = 1,
  SK_V1_DIRECTORY_SYNC_UNCONFIRMED = 2, SK_V1_PROVIDER_CONFIRMED = 3
};
typedef struct {
  SketchProjectionV1 projection;
  SketchEffectTokenV1 effect;
  SketchFailureV1 failure;
  uint64_t operation_id, captured_session, captured_revision;
  uint64_t written_destination, written_revision;
  uint32_t effect_kind, change_flags, invalidation, save_purpose;
  uint32_t has_save_result, save_result, acknowledgement, durability;
  uint32_t retry_action, reserved; /* Rust-selected command tag; 0 = no retry */
} SketchOutcomeV1;
/* Status != OK rejects without consuming input/result ownership. Status OK may carry
 * an accepted failure effect. Projection identifies the still-actionable continuation.
 * Native text validation runs before explicit file/navigation commands. */
#endif
