#ifndef NATIVE_SKETCH_V2_H
#define NATIVE_SKETCH_V2_H
#include <stddef.h>
#include <stdint.h>
#ifdef __cplusplus
extern "C" {
#endif
typedef uint64_t NsV2Handle;
typedef struct { uint8_t bytes[72]; } NsV2Error;
/* Additive ABI. No v1 handle or packet is interchangeable with v2.
 * Status 0 means transport accepted; inspect the reply for application failure.
 * Other statuses transfer nothing. Handles are never pointers; release once.
 * Pointers/ranges must be live, aligned and pairwise disjoint. Null bytes only
 * when length is zero. Output scalar/handle slots must initially be zero.
 * Invalid nonnull foreign pointers are undefined caller behavior. No pointer is
 * retained. Error is mandatory writable storage, filled without allocation.
 * App calls belong to the creating thread. Work/results move serially between
 * workers and owner; no concurrent same-handle access. Buffers can move threads.
 * See docs/architecture/022-portable-exports.md for complete lifecycle rules. */
uint32_t ns_v2_app_new(const uint8_t *, size_t, NsV2Handle *, NsV2Error *);
uint32_t ns_v2_app_call(NsV2Handle, const uint8_t *, size_t, NsV2Handle *, NsV2Error *);
uint32_t ns_v2_app_take_work(NsV2Handle, const uint8_t *, size_t, NsV2Handle *, NsV2Handle *, NsV2Error *);
uint32_t ns_v2_work_copy_input(NsV2Handle, const uint8_t *, size_t, NsV2Handle *, NsV2Error *);
uint32_t ns_v2_work_call(NsV2Handle *, const uint8_t *, size_t, NsV2Handle *, NsV2Handle *, NsV2Handle *, NsV2Error *);
uint32_t ns_v2_app_complete(NsV2Handle, const uint8_t *, size_t, NsV2Handle *, NsV2Handle *, NsV2Error *);
uint32_t ns_v2_input_free(NsV2Handle *, NsV2Error *);
uint32_t ns_v2_buffer_len(NsV2Handle, size_t *, NsV2Error *);
uint32_t ns_v2_buffer_copy(NsV2Handle, size_t, uint8_t *, size_t, size_t *, NsV2Error *);
uint32_t ns_v2_buffer_free(NsV2Handle *, NsV2Error *);
uint32_t ns_v2_app_free(NsV2Handle *, NsV2Error *);
/* No active-work/result-free. Only a returned result after proven owner exit
 * may be retired. This is not a save acknowledgment or provider grant release. */
uint32_t ns_v2_result_retire_stopped(NsV2Handle *, NsV2Error *);
#ifdef __cplusplus
}
#endif
#endif
