#include "support.h"
static void number(uint8_t *p, double value) {
  uint64_t bits;
  memcpy(&bits, &value, 8);
  put64(p, bits);
}
static size_t viewport_call(TestApp a, uint32_t op, const uint8_t *b, size_t n,
                            uint32_t expected, uint8_t *out) {
  uint8_t packet[16424];
  NsV2Handle reply = 0;
  NsV2Error error;
  size_t length = frame(packet, op, b, n), written = 0, count = 0;
  assert(ns_v2_app_call(a.app, packet, length, &reply, &error) == expected);
  if (expected) {
    assert(reply == 0);
    return 0;
  }
  assert(ns_v2_buffer_len(reply, &count, &error) == 0 && count <= 65616);
  assert(ns_v2_buffer_copy(reply, 0, out, count, &written, &error) == 0);
  assert(written == count && ns_v2_buffer_free(&reply, &error) == 0);
  return count;
}
static void draw_long(TestApp a) {
  uint8_t b[14440] = {0}, r[512];
  context(a, b);
  put32(b + 24, 6);
  put64(b + 32, UINT64_C(0xfedcba9876543210));
  put32(b + 40, 2);
  call(a, 10, b, 48, r);
  memset(b, 0, sizeof(b));
  context(a, b);
  put64(b + 24, UINT64_C(0xfedcba9876543210));
  put32(b + 32, 300);
  for (size_t i = 0; i < 300; i++) {
    uint8_t *s = b + 40 + 48 * i;
    put64(s + 8, UINT64_C(0xf000000000000000) + i);
    number(s + 16, i % 2 ? 999999010.0 : 0.0);
    number(s + 24, 20.0);
    number(s + 32, (double)i);
  }
  viewport_call(a, 12, b, sizeof(b), 0, r);
  memset(b, 0, 48);
  context(a, b);
  put32(b + 24, 7);
  put64(b + 32, UINT64_C(0xfedcba9876543210));
  call(a, 10, b, 40, r);
}
void check_viewport(void) {
  TestApp a = create(), foreign = create();
  draw_long(a);
  uint8_t b[64] = {0}, r[65616], request[128];
  NsV2Error error;
  put64(b, a.session);
  number(b + 16, 999999000.0);
  number(b + 24, 0.0);
  number(b + 32, 100.0);
  number(b + 40, 100.0);
  number(b + 48, 2.0);
  call(a, 40, b, 56, r);
  uint64_t scene = get64(r + 72);
  memset(b, 0, sizeof(b));
  put64(b, scene);
  put32(b + 16, 133);
  put32(b + 20, 137);
  put32(b + 8, 7);
  viewport_call(a, 45, b, 24, 1, r);
  put32(b + 8, 0);
  NsV2Handle held[64] = {0};
  size_t n = frame(request, 5, NULL, 0);
  for (size_t i = 0; i < 64; i++)
    assert(ns_v2_app_call(a.app, request, n, &held[i], &error) == 0);
  viewport_call(a, 45, b, 24, 3, r);
  for (size_t i = 0; i < 64; i++)
    assert(ns_v2_buffer_free(&held[i], &error) == 0);
  assert(viewport_call(a, 45, b, 24, 0, r) == 136);
  assert(get32(r + 72) == 1 && get32(r + 76) == 0 && get64(r + 88) == scene);
  assert(get32(r + 128) == 133 && get32(r + 132) == 137);
  uint64_t prepared = get64(r + 80), count = get64(r + 120);
  assert(count > 744);
  viewport_call(a, 45, b, 24, 9, r);
  put64(b, scene);
  call(a, 44, b, 8, r);
  put64(b, prepared);
  put32(b + 8, 0);
  put32(b + 12, 744);
  viewport_call(foreign, 46, b, 16, 7, r);
  uint64_t total = 0;
  while (total < count) {
    put32(b + 8, (uint32_t)total);
    size_t length = viewport_call(a, 46, b, 16, 0, r);
    uint32_t copied = get32(r + 72);
    assert(copied > 0 && copied <= 744);
    assert(length == 80 + copied * 88);
    total += copied;
  }
  assert(total == count);
  put32(b + 8, (uint32_t)count);
  viewport_call(a, 46, b, 16, 0, r);
  assert(get32(r + 72) == 0);
  put32(b + 8, (uint32_t)count + 1);
  viewport_call(a, 46, b, 16, 1, r);
  memset(b, 0, sizeof(b));
  context(a, b);
  put32(b + 24, 6);
  call(a, 20, b, 32, r);
  assert(get32(r + 232) == 3);
  uint64_t effect = get64(r + 240);
  memset(b, 0, sizeof(b));
  context(a, b);
  put64(b + 24, effect);
  put32(b + 32, 3);
  call(a, 21, b, 40, r);
  assert(get32(r + 232) == 6);
  effect = get64(r + 240);
  context(a, b);
  put64(b + 24, effect);
  put32(b + 32, 5);
  call(a, 21, b, 40, r);
  assert(get32(r + 232) == 7);
  put64(b, a.session);
  call(a, 2, b, 8, r);
  for (;;) {
    NsV2Handle work = 0, reply = 0, result = 0, input = 0;
    n = frame(request, 30, NULL, 0);
    assert(ns_v2_app_take_work(a.app, request, n, &work, &reply, &error) == 0);
    copy_reply(reply, r);
    reply = 0;
    if (!work)
      break;
    n = frame(request, 106, NULL, 0);
    assert(ns_v2_work_call(&work, request, n, &input, &result, &reply,
                           &error) == 0);
    copy_reply(reply, r);
    reply = 0;
    n = frame(request, 31, NULL, 0);
    assert(ns_v2_app_complete(a.app, request, n, &result, &reply, &error) == 0);
    copy_reply(reply, r);
  }
  assert(ns_v2_app_free(&a.app, &error) == 9);
  put64(b, prepared);
  viewport_call(foreign, 47, b, 8, 7, r);
  call(a, 47, b, 8, r);
  viewport_call(a, 47, b, 8, 7, r);
  assert(ns_v2_app_free(&a.app, &error) == 0);
  close_app(foreign);
  puts("PASS native viewport: actual outlines, pages, source release, "
       "ownership, capacity, shutdown");
}
