#ifndef RENDERER_COMPARISON_H
#define RENDERER_COMPARISON_H
#include <stdint.h>
/* Experiment-only. All calls serialized on one host worker queue. Strings borrowed for
   call; returned error string valid until next call on that thread. create retains layer;
   caller owns handle until destroy. Status 0 success, 1 failure. No Rust unwind crosses ABI. */
void *rc_create(const char *config_json, void *ca_metal_layer);
int32_t rc_step(void *handle, uint32_t phase, uint64_t frame_index);
int32_t rc_finish(void *handle, const char *status, double measured_elapsed_s);
void rc_destroy(void *handle);
const char *rc_error(void *handle);
/* Experiment-only native fixture bridge. All calls serialized. Returned arrays are
   borrowed until prepare/destroy. Returned strings until next string-returning call.
   A failed fixture may only be destroyed. See coregraphics-appkit-contract.md. */
typedef struct { uint32_t verb, reserved; double points[6]; } RcCommand;
typedef struct { uint64_t stroke_id, command_start, command_count; double rgba[4]; } RcOutline;
typedef struct {
    const RcCommand *commands; uint64_t command_count;
    const RcOutline *outlines; uint64_t outline_count;
    const RcCommand *clip_commands; uint64_t clip_command_count;
    uint32_t has_clip, geometry_changed;
    double transform[6], image_transform[6];
    double input_ms, outline_ms, export_ms;
} RcFixtureFrame;
void *rc_fixture_create(const char *fixture, uint32_t width, uint32_t height);
int32_t rc_fixture_prepare(void *handle, uint64_t frame, RcFixtureFrame *out);
const char *rc_fixture_metadata_json(void *handle);
const char *rc_fixture_state_hash(void *handle, uint64_t frame, uint32_t actual);
const uint8_t *rc_fixture_image_rgba(void *handle, uint64_t *byte_count);
void rc_fixture_destroy(void *handle);
#endif
