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
#endif
