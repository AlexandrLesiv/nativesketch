#ifndef PEN_COMPARISON_H
#define PEN_COMPARISON_H
#include "renderer_comparison.h"
/* Experiment ABI 1. All calls (including destruction) serialized on the host main
   thread. Unique handles must be destroyed once. Returned arrays are immutable and
   borrowed until the next advance/control/destroy on that replay. Strings until next
   metadata call/destroy. No Rust unwind crosses ABI; failure poisons the handle.
   Status 0 success, 1 failure. pc_error is borrowed until next failure on this thread. */
typedef struct {
    const RcCommand *base_commands; uint64_t base_command_count;
    const RcOutline *base_outlines; uint64_t base_outline_count;
    const RcCommand *live_commands; uint64_t live_command_count;
    const RcOutline *live_outlines; uint64_t live_outline_count;
    const RcCommand *clip_commands; uint64_t clip_command_count;
    const uint8_t *image_rgba; uint64_t image_byte_count;
    uint64_t document_revision, live_revision, viewport_revision;
    uint64_t last_sample_id, total_samples, delivered_now, expected_samples;
    double last_sample_time_s, first_delivered_time_s, last_delivered_time_s;
    double input_ms, outline_ms, export_ms;
    double transform[6], image_transform[6];
    uint32_t has_clip, has_image, trace_complete, cache_allowed;
    char geometry_hash[65], document_hash[65];
} PcSnapshot;
/* Disjoint CPU scopes: path conversion; background allocation/paint/snapshot;
   target paint+flush/submit. Does not include drawable acquisition/presentation.
   Cache bytes count full-viewport RGBA allocation intent, excludes library overhead. */
typedef struct {
    double path_ms, cache_build_ms, paint_submit_ms;
    uint64_t cache_bytes;
    uint32_t cache_rebuilt, reserved;
} PcPaintMetrics;
/* config: scene empty/dense1000/dense5000/alpha/clip/image; trace short/long/panzoom;
   width,height physical pixels; duration_s (0,600]; sample_hz [30,1000].
   Generated traces are synthetic, not recorded Wacom input. */
void *pc_replay_create(const char *config_json);
/* elapsed_s is monotonic wall-clock offset. Applies ALL due samples. Never advances
   by rendered frames. Final advance >=duration_s commits final real sample. */
int32_t pc_replay_advance(void *handle, double elapsed_s, PcSnapshot *out);
/* Untimed diagnostics: 0 cancel, 1 undo, 2 redo. Stop replay after control actions. */
int32_t pc_replay_control(void *handle, uint32_t action, PcSnapshot *out);
const char *pc_replay_metadata_json(void *handle);
void pc_replay_destroy(void *handle);
/* device/queue retained by Skia context; caller must also keep them alive until
   destroy. texture is borrowed for render and retained by Skia as required for GPU
   work; host retains drawable through presentation/completion. BGRA8Unorm,
   framebufferOnly=false, exact width/height. Skia submits onto supplied queue;
   host enqueues presentation on that same queue after return, without CPU wait.
   policy: 0 retained vectors R, 1 cached static background C. */
void *pc_skia_create(void *device, void *queue, uint32_t width, uint32_t height);
int32_t pc_skia_render(void *handle, void *texture, const PcSnapshot *snapshot,
                       uint32_t policy, PcPaintMetrics *out);
/* Clear path/image/background caches between replay handles; retain Metal context. */
int32_t pc_skia_reset(void *handle);
void pc_skia_destroy(void *handle);
const char *pc_error(void);
#endif
