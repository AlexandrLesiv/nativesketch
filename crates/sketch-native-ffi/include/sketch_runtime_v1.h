#ifndef SKETCH_RUNTIME_V1_H
#define SKETCH_RUNTIME_V1_H
#include "sketch_commands_v1.h"
#ifdef __cplusplus
extern "C" {
#endif
/* Zero limits select defaults. struct_size is sizeof this v1 config. Empty roots disable
 * that service observably; root I/O/locking is asynchronous and never blocks creation. */
typedef struct {
  uint32_t struct_size, max_sessions, max_operations, max_events;
  uint64_t snapshot_bytes, build_id;
  SketchTextV1 recovery_root, diagnostics_root; /* copied; local absolute UTF-8 paths */
} SketchRuntimeConfigV1;
typedef struct {
  uint64_t grant_id;
  SketchTextV1 path; /* copied, max16KiB; no NUL; trusted local-filesystem adapter only */
  uint32_t provider, reserved; /* provider0 local; others return UNSUPPORTED */
} SketchLocationV1;
typedef struct {
  uint64_t operation_id, event_id, window, session, request, grant_id;
  uint32_t kind, reserved;
  SketchFailureV1 failure;
} SketchEventV1;
enum SketchEventKindV1 {
  SK_V1_OWNER_READY = 1, SK_V1_GRANT_RELEASE = 2, SK_V1_RECOVERY_READY = 3,
  SK_V1_MAINTENANCE_COMPLETE = 4, SK_V1_RUNTIME_FAILURE = 5, SK_V1_SHUTDOWN_COMPLETE = 6
};
typedef struct {
  uint32_t pending_operations, ready_events, retained_events, attached_sessions;
  uint32_t needs_poll, needs_idle, stopping, stopped;
  uint64_t wake_after_ms, dropped_diagnostics, failed_diagnostic_sinks; /* UINT64_MAX: no timer */
} SketchRuntimeStateV1;
uint32_t sketch_runtime_v1_version(void);
uint32_t sketch_runtime_v1_create(const SketchRuntimeConfigV1 *, SketchRuntimeV1 **out_runtime);
uint32_t sketch_runtime_v1_state(SketchRuntimeV1 *, SketchRuntimeStateV1 *out_state);
/* At most32 metadata notifications per call; no encode, decode, snapshot or filesystem work.
 * Metadata delivered once; payload remains until apply/release. capacity0 is query-only.
 * Writes min(capacity,32,ready) events; count is actual, ready_events is remaining.
 * Poll never invokes native callbacks. */
uint32_t sketch_runtime_v1_poll(SketchRuntimeV1 *, SketchEventV1 *, uint32_t capacity,
                              uint32_t *out_count, SketchRuntimeStateV1 *out_state);
/* Runs at most one due owner-side admission outside input callbacks; active ink defers it. */
uint32_t sketch_runtime_v1_idle(SketchRuntimeV1 *, SketchRuntimeStateV1 *out_state);
/* Release notifications/inventory or abandon resolution/read candidates off-owner.
 * Abandon cancels its matching continuation only; never discards attached save receipts. */
uint32_t sketch_runtime_v1_release_event(SketchRuntimeV1 *, uint64_t event_id);
uint32_t sketch_runtime_v1_request_shutdown(SketchRuntimeV1 *);
/* BUSY retains pointer until sessions detached, issued events settled and worker stopped. */
uint32_t sketch_runtime_v1_free(SketchRuntimeV1 **runtime);
#ifdef __cplusplus
}
#endif
#endif
