#ifndef SKETCH_RECOVERY_V1_H
#define SKETCH_RECOVERY_V1_H
#include "sketch_runtime_v1.h"
#ifdef __cplusplus
extern "C" {
#endif
typedef struct {
  uint64_t item_id, generation, revision, created_seconds;
  uint8_t recovery_id[16];
  uint32_t display_name_length, rejected_generations;
  uint8_t display_name[256]; /* private local UI metadata, never diagnostic content */
} SketchRecoveryItemV1;
typedef struct {
  /* Worker snapshot. Capacity means no identity slot or exhausted byte/reservation
   * budget (including bounded inventory overflow), not a guarantee the next file fits.
   * Usage includes resolved/empty identities, generations, markers and staging files.
   * usage_known=0 means unsafe/unreadable/incomplete accounting; used_bytes is then0.
   * A valid empty root has usage_known=1 and used_bytes=0. Refresh after maintenance. */
  uint32_t count, rejected_sessions, invalid_markers, capacity_blocked, usage_known, reserved;
  uint64_t used_bytes, limit_bytes;
} SketchRecoveryInfoV1;
enum SketchMaintenanceTagV1 {
  SK_V1_RECOVERY_LIST = 1, SK_V1_RECOVERY_DISCARD = 2,
  SK_V1_RECOVERY_RETRY = 3, SK_V1_DIAGNOSTICS_EXPORT = 4
};
typedef struct {
  uint32_t tag, reserved;
  uint64_t item_id, window; /* RETRY: item_id=0; window=0 all or stable window ID */
  SketchLocationV1 location; /* used only for diagnostics export */
} SketchMaintenanceV1;
/* DISCARD is called only after native confirmation of selected runtime-issued item.
 * It validates identity/generation and never erases a newer unseen generation. */
uint32_t sketch_runtime_v1_maintenance(SketchRuntimeV1 *, uint64_t request,
                                     const SketchMaintenanceV1 *, uint64_t *out_operation);
/* Count/copy frozen inventory; insufficient capacity writes count only, no partial items. */
uint32_t sketch_runtime_v1_recovery_copy(SketchRuntimeV1 *, uint64_t event_id,
  SketchRecoveryItemV1 *, uint32_t capacity, uint32_t *out_count, SketchRecoveryInfoV1 *out_info);
#ifdef __cplusplus
}
#endif
#endif
