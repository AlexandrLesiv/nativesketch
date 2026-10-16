#include "native_sketch_v2.h"
#include <assert.h>
#include <string.h>
#include <stdio.h>
#include <pthread.h>
typedef struct { NsV2Handle app; uint64_t session; } TestApp;
void put32(uint8_t *, uint32_t);
void put64(uint8_t *, uint64_t);
uint32_t get32(const uint8_t *);
uint64_t get64(const uint8_t *);
size_t frame(uint8_t *, uint32_t, const uint8_t *, size_t);
size_t copy_reply(NsV2Handle, uint8_t *);
void call(TestApp, uint32_t, const uint8_t *, size_t, uint8_t *);
TestApp create(void);
void context(TestApp, uint8_t *);
void close_app(TestApp);
uint64_t queue_save(TestApp);
void check_worker_roundtrip(void);
TestApp create_policy(uint32_t);
void check_bounds(TestApp);
void check_bound_chooser(TestApp);
