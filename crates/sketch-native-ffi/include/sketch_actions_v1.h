#ifndef SKETCH_ACTIONS_V1_H
#define SKETCH_ACTIONS_V1_H
#include "sketch_recovery_v1.h"
#ifdef __cplusplus
extern "C" {
#endif
/* Separate high-level intent tags; legacy primitive tags remain unchanged. */
enum {
  SK_V1_ACTION_CHOOSE_TOOL = 1, SK_V1_ACTION_CYCLE_SELECTION = 2,
  SK_V1_ACTION_DELETE = 3, SK_V1_ACTION_UNDO = 4, SK_V1_ACTION_REDO = 5,
  SK_V1_ACTION_SCALE_SELECTION = 6
};
enum { SK_V1_CYCLE_PREVIOUS = 1, SK_V1_CYCLE_NEXT = 2 };
enum { SK_V1_ANCHOR_SELECTION_CENTER = 1 };
typedef struct {
  uint32_t tag, option;
  double factor;
} SketchEditorActionV1;
/* CHOOSE_TOOL option=PEN/SELECT; CYCLE option=PREVIOUS/NEXT; SCALE option=CENTER
 * and factor finite (0,100]. All unused fields are zero. Unknown tags/options reject.
 * Validate before cancel: rejected intents preserve draft/preview/settings/history.
 * Success cancels the interaction then applies once, returning one final projection.
 * UI Undo intentionally cancels+undoes; primitive Undo still rejects active interaction.
 * Same owner-thread, panic containment and borrowed-call buffer rules as session v1.
 * Invalid calls preserve output. Scale uses committed bounds, not transform preview. */
uint32_t sketch_session_v1_action(SketchRuntimeV1 *, SketchSessionV1 *, SketchContextV1,
                                 const SketchEditorActionV1 *, SketchOutcomeV1 *);
/* Advisory history/selection/capability check ignoring active draft; execution additionally
 * checks context, transform bounds and capacity. Existing disabled reason numbers apply. */
uint32_t sketch_session_v1_action_availability(SketchRuntimeV1 *, SketchSessionV1 *,
                                             const SketchEditorActionV1 *, SketchAvailabilityV1 *);
#ifdef __cplusplus
}
#endif
#endif
