#ifndef SKETCH_SESSION_V1_H
#define SKETCH_SESSION_V1_H
#include "sketch_recovery_v1.h"
#include "sketch_actions_v1.h"
#ifdef __cplusplus
extern "C" {
#endif
typedef struct {
  uint32_t capabilities, tool; /* capability masks EDIT=1, EXPORT=2; PEN=1, SELECT=2 */
  SketchBrushV1 brush;
} SketchSessionConfigV1;
uint32_t sketch_session_v1_create(SketchRuntimeV1 *, const SketchSessionConfigV1 *, SketchSessionV1 **);
uint32_t sketch_session_v1_projection(SketchRuntimeV1 *, SketchSessionV1 *, SketchProjectionV1 *);
uint32_t sketch_session_v1_availability(SketchRuntimeV1 *, SketchSessionV1 *, uint32_t action,
                                      SketchAvailabilityV1 *);
uint32_t sketch_session_v1_dispatch(SketchRuntimeV1 *, SketchSessionV1 *, SketchContextV1,
                                  const SketchCommandV1 *, SketchOutcomeV1 *);
uint32_t sketch_session_v1_input(SketchRuntimeV1 *, SketchSessionV1 *, SketchContextV1,
  uint64_t pointer, const SketchSampleV1 *, uint32_t count, SketchOutcomeV1 *);
uint32_t sketch_session_v1_reply(SketchRuntimeV1 *, SketchSessionV1 *, SketchContextV1,
                               SketchEffectTokenV1, uint32_t reply, SketchOutcomeV1 *);
/* Copies descriptor and reserves ordered barrier; no filesystem work or snapshot yet.
 * Grant ownership transfers only on OK and ends with GRANT_RELEASE notification. */
uint32_t sketch_session_v1_supply_location(SketchRuntimeV1 *, SketchSessionV1 *, SketchContextV1,
  SketchEffectTokenV1, const SketchLocationV1 *, uint64_t *out_operation);
/* Applies issued result, never caller-authored success. BUSY retains result during ink.
 * Resolution admission/candidate installation run only in owner idle callbacks. */
uint32_t sketch_session_v1_apply_event(SketchRuntimeV1 *, SketchSessionV1 *, uint64_t event_id,
                                     SketchOutcomeV1 *);
/* NOT_FOUND when no selection; output unchanged on rejection. */
uint32_t sketch_session_v1_selected_bounds(SketchRuntimeV1 *, SketchSessionV1 *, SketchBoundsV1 *);
uint32_t sketch_session_v1_scene(SketchRuntimeV1 *, SketchSessionV1 *, SketchSceneV1 **);
uint32_t sketch_scene_v1_info(SketchSceneV1 *, SketchSceneInfoV1 *);
uint32_t sketch_scene_v1_copy(SketchSceneV1 *, SketchPathV1 *, uint32_t capacity, uint32_t *out_count);
uint32_t sketch_scene_v1_free(SketchSceneV1 **);
/* Host supplies fresh secure random128-bit ID per document session; filesystem allocation
 * is asynchronous. New/import/recovery requires registration for the new session ID. */
uint32_t sketch_session_v1_recovery_identity(SketchRuntimeV1 *, SketchSessionV1 *,
  uint64_t expected_session, const uint8_t identity[16], SketchTextV1 display_name);
/* After confirmed close, pristine never-edited initialization, or engine poisoning: transfers document to bounded retirement.
 * BUSY preserves handle. Worker grants/receipts remain app-runtime owned after detach. */
uint32_t sketch_session_v1_detach(SketchRuntimeV1 *, SketchSessionV1 **);
#ifdef __cplusplus
}
#endif
#include "sketch_renderer_v1.h"
#endif
