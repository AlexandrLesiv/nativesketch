#include "sketch_session_v1.h"
#include <assert.h>
#include <stddef.h>
#include <stdio.h>
#include <unistd.h>
_Static_assert(sizeof(SketchSampleV1) == 48, "sample layout");
_Static_assert(sizeof(SketchCommandV1) == 48, "command layout");
_Static_assert(offsetof(SketchOutcomeV1, effect) == sizeof(SketchProjectionV1), "projection layout");
int main(void) {
  SketchRuntimeConfigV1 config = {0};
  config.struct_size = sizeof(config);
  SketchRuntimeV1 *runtime = NULL;
  assert(sketch_runtime_v1_create(&config, &runtime) == SK_V1_OK);
  SketchSessionConfigV1 session_config = {SK_V1_CAP_EDIT | SK_V1_CAP_EXPORT,
      SK_V1_PEN, {0, 0, 0, 1, 4}};
  SketchSessionV1 *session = NULL;
  assert(sketch_session_v1_create(runtime, &session_config, &session) == SK_V1_OK);
  SketchProjectionV1 projection = {0};
  assert(sketch_session_v1_projection(runtime, session, &projection) == SK_V1_OK);
  SketchContextV1 context = {projection.session, 9, 0};
  SketchCommandV1 command = {0};
  command.tag = SK_V1_BEGIN_STROKE;
  command.payload.stroke.pointer = 2;
  command.payload.stroke.device = SK_V1_STYLUS;
  SketchOutcomeV1 result = {0};
  assert(sketch_session_v1_dispatch(runtime, session, context, &command, &result) == SK_V1_OK);
  context.interaction = result.projection.interaction;
  SketchSampleV1 samples[2] = {{1, 1, 1, 0.5, 1, 1, 0}, {2, 2, 2, 0.5, 2, 1, 99}};
  uint64_t generation = result.projection.scene_generation;
  assert(sketch_session_v1_input(runtime, session, context, 2, samples, 2, &result) == SK_V1_INVALID_ARGUMENT);
  assert(sketch_session_v1_projection(runtime, session, &projection) == SK_V1_OK);
  assert(projection.scene_generation == generation && projection.stroke_count == 0);
  command.tag = SK_V1_CANCEL_INTERACTION;
  assert(sketch_session_v1_dispatch(runtime, session, context, &command, &result) == SK_V1_OK);
  assert(sketch_session_v1_detach(runtime, &session) == SK_V1_OK);
  assert(session == NULL);
  assert(sketch_runtime_v1_request_shutdown(runtime) == SK_V1_OK);
  for (int attempt = 0; attempt < 5000 && runtime; attempt++) {
    SketchEventV1 events[32];
    SketchRuntimeStateV1 state = {0};
    uint32_t count = 0;
    assert(sketch_runtime_v1_poll(runtime, events, 32, &count, &state) == SK_V1_OK);
    for (uint32_t index = 0; index < count; index++) {
      assert(sketch_runtime_v1_release_event(runtime, events[index].event_id) == SK_V1_OK);
    }
    uint32_t freed = sketch_runtime_v1_free(&runtime);
    assert(freed == SK_V1_OK || freed == SK_V1_BUSY);
    usleep(1000);
  }
  assert(runtime == NULL);
  puts("native C layout, atomic invalid batch, lifecycle passed");
  return 0;
}
