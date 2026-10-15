#ifndef SKETCH_RENDERER_V1_H
#define SKETCH_RENDERER_V1_H
#include "sketch_values_v1.h"
#ifdef __cplusplus
extern "C" {
#endif
typedef struct SketchRendererV1 SketchRendererV1;
typedef struct {
  uint32_t struct_size, focused;
  uint64_t session, revision, scene_generation, surface_generation;
  double logical_width, logical_height;
  uint32_t pixel_width, pixel_height;
  double zoom, offset_x, offset_y;
  double selection_rgba[4], focus_rgba[4];
} SketchRenderRequestV1;
typedef struct {
  uint64_t context_generation, scene_prepare_ns, render_submit_ns;
  uint64_t geometry_bytes, path_bytes, background_bytes;
  uint64_t path_hits, path_misses;
  uint32_t background_hit, reserved;
} SketchRenderStatsV1;
/* macOS only; other hosts return UNSUPPORTED. All calls use the creating thread.
 * Device/queue are valid borrowed Objective-C Metal objects, retained by the renderer.
 * Swift owns presentation and uses this same queue after a successful draw. */
uint32_t sketch_renderer_v1_create(const void *device, const void *queue,
                                  SketchRendererV1 **out_renderer);
/* texture is the current drawable's BGRA8Unorm, single-sample Metal texture.
 * Its dimensions equal request pixel dimensions; the host keeps the drawable alive
 * through presentation. Rendering submits without waiting for GPU completion.
 * All scalar fields/colors must be finite; focused is 0/1 and struct_size must match.
 * Expected scene identity must match the session. Surface generations never decrease.
 * Stats are written only on success. Errors never replace the presented frame.
 * No caller-owned pointer is retained except create's explicitly retained objects. */
uint32_t sketch_renderer_v1_draw(SketchRuntimeV1 *, SketchSessionV1 *, SketchRendererV1 *,
  const void *texture, const SketchRenderRequestV1 *, SketchRenderStatsV1 *);
/* Drops disposable caches and advances context generation. Does not mutate editor state. */
uint32_t sketch_renderer_v1_reset(SketchRendererV1 *);
/* Null handle is allowed; on success clears the caller's pointer. */
uint32_t sketch_renderer_v1_free(SketchRendererV1 **);
#ifdef __cplusplus
}
#endif
#endif
