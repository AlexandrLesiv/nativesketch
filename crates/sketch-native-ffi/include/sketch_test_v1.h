#ifndef SKETCH_TEST_V1_H
#define SKETCH_TEST_V1_H
#include "sketch_session_v1.h"
/* Available only in builds explicitly enabling native-test-controls. Not included by
 * production umbrella. Controls affect real worker boundaries, never editor commands. */
#ifdef __cplusplus
extern "C" {
#endif
enum SketchTestStageV1 {
  SK_V1_TEST_RESOLVE = 1, SK_V1_TEST_BEFORE_CREATE = 2, SK_V1_TEST_AFTER_WRITE = 3,
  SK_V1_TEST_AFTER_FILE_SYNC = 4, SK_V1_TEST_BEFORE_REPLACE = 5,
  SK_V1_TEST_AFTER_REPLACE = 6, SK_V1_TEST_DIRECTORY_SYNC = 7,
  SK_V1_TEST_AFTER_READ_VALIDATION = 8
};
enum SketchTestModeV1 { SK_V1_TEST_HOLD = 1, SK_V1_TEST_FAIL = 2 };
typedef struct {
  SketchTextV1 disposable_control_root;
  uint32_t stage, mode;
} SketchTestControlV1;
/* One-shot next matching boundary. Root must be disposable and empty at registration.
 * Worker creates reached marker; external test writes release marker for HOLD.
 * HOLD aborts at60s or shutdown, reporting stage-appropriate failure/uncertainty.
 * Storage stages match explicit Save/SaveAs only; RESOLVE matches user source/destination.
 * AFTER_READ_VALIDATION matches successful Document/Recovery validation only, before
 * candidate publication. New/invalid reads do not consume it. FAIL reports Access.
 * No production environment variable can enable these symbols or behavior. */
uint32_t sketch_runtime_v1_test_arm(SketchRuntimeV1 *, const SketchTestControlV1 *);
#ifdef __cplusplus
}
#endif
#endif
