#ifndef SKETCH_VALUES_V1_H
#define SKETCH_VALUES_V1_H
#include <stdint.h>
#include "sketch_tags_v1.h"
typedef struct SketchRuntimeV1 SketchRuntimeV1;
typedef struct SketchSessionV1 SketchSessionV1;
typedef struct SketchSceneV1 SketchSceneV1;
/* Independent native ABI v1. All enum values are fixed; unknown tags are rejected. */
enum SketchStatusV1 {
  SK_V1_OK = 0, SK_V1_INVALID_ARGUMENT = 1, SK_V1_INVALID_STATE = 2,
  SK_V1_CAPACITY = 3, SK_V1_IO = 4, SK_V1_DOCUMENT = 5, SK_V1_PANIC = 6,
  SK_V1_STALE = 7, SK_V1_DENIED = 8, SK_V1_BUSY = 9, SK_V1_POISONED = 10,
  SK_V1_WRONG_OWNER = 11, SK_V1_STOPPED = 12, SK_V1_UNSUPPORTED = 13,
  SK_V1_NOT_FOUND = 14
};
enum SketchDisabledReasonV1 {
  SK_V1_ENABLED = 0, SK_V1_READ_ONLY = 1, SK_V1_INTERACTION_ACTIVE = 2,
  SK_V1_WRONG_TOOL = 3, SK_V1_NOTHING_SELECTED = 4, SK_V1_HISTORY_EMPTY = 5,
  SK_V1_NO_ACTIVE_STROKE = 6, SK_V1_NO_ACTIVE_TRANSFORM = 7,
  SK_V1_ACTION_BUSY = 8, SK_V1_EXPORT_DENIED = 9, SK_V1_ENGINE_UNAVAILABLE = 10
};
enum SketchFilePhaseV1 {
  SK_V1_FILE_IDLE = 0, SK_V1_FILE_DESTINATION = 1, SK_V1_FILE_SOURCE = 2,
  SK_V1_FILE_UNSAVED = 3, SK_V1_FILE_WRITES = 4, SK_V1_FILE_REPLACEMENT = 5,
  SK_V1_FILE_CLOSE_READY = 6
};
enum SketchFailureDomainV1 {
  SK_V1_FAILURE_NONE = 0, SK_V1_FAILURE_EDITOR = 1, SK_V1_FAILURE_ADMISSION = 2,
  SK_V1_FAILURE_WRITE = 3, SK_V1_FAILURE_ROUTER = 4, SK_V1_FAILURE_RUNTIME = 5,
  SK_V1_FAILURE_RECOVERY = 6
};
typedef struct { const uint8_t *bytes; uint32_t length; } SketchTextV1;
typedef struct { double red, green, blue, alpha, width; } SketchBrushV1;
typedef struct { double x, y, width, height; } SketchBoundsV1;
typedef struct {
  uint64_t session, request, interaction; /* interaction 0 = none */
} SketchContextV1;
typedef struct {
  uint64_t owner, session, serial, request; /* owner is stable window ID, 0 = runtime */
} SketchEffectTokenV1;
typedef struct {
  double x, y, timestamp, pressure;
  uint64_t sample_id;
  uint32_t has_pressure, kind; /* actual0, predicted1, correction2 */
} SketchSampleV1;
typedef struct {
  uint64_t stroke_id;
  uint32_t verb, reserved; /* move0, line1, cubic2, close3 */
  double red, green, blue, alpha;
  double x1, y1, x2, y2, x3, y3;
} SketchPathV1;
typedef struct {
  uint64_t window, session, revision, scene_generation, interaction;
  uint64_t stroke_count, selected_id, destination, binding_generation, binding_grant;
  SketchEffectTokenV1 continuation;
  SketchBrushV1 brush;
  uint32_t tool, interaction_kind, capabilities, dirty;
  uint32_t can_undo, can_redo, file_phase, navigation;
  uint32_t outstanding_saves, preparing_operations, recovery_state, reserved;
} SketchProjectionV1;
typedef struct {
  uint32_t enabled, reason; /* reason: fixed native failure/disabled-reason code */
} SketchAvailabilityV1;
typedef struct {
  uint64_t session, revision, generation, command_count;
} SketchSceneInfoV1;
typedef struct {
  uint32_t domain, code, operation, reserved;
  uint64_t session, request, operation_id;
} SketchFailureV1;
#endif
