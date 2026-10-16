#include "support.h"
static void floating(uint8_t *p, double value) {
  uint64_t bits;
  memcpy(&bits, &value, 8);
  put64(p, bits);
}
static double number(const uint8_t *p) {
  uint64_t bits = get64(p);
  double value;
  memcpy(&value, &bits, 8);
  return value;
}
static void edit_tag(TestApp a, uint32_t tag, uint8_t *r) {
  uint8_t b[32] = {0};
  context(a, b);
  put32(b + 24, tag);
  call(a, 10, b, sizeof(b), r);
}
static void undo(TestApp a, uint8_t *r) {
  uint8_t b[32] = {0};
  context(a, b);
  put32(b + 24, 4);
  call(a, 11, b, sizeof(b), r);
}
static void bounds(TestApp a, double extent) {
  uint8_t b[8], r[512];
  put64(b, a.session);
  call(a, 6, b, 8, r);
  assert(number(r + 72) == 19 && number(r + 80) == 29);
  assert(number(r + 88) == extent && number(r + 96) == extent);
}
static void seed(TestApp a) {
  uint8_t b[96] = {0}, r[512];
  context(a, b);
  put32(b + 24, 6);
  put64(b + 32, 7);
  put32(b + 40, 2);
  call(a, 10, b, 48, r);
  memset(b, 0, sizeof(b));
  context(a, b);
  put64(b + 24, 7);
  put32(b + 32, 1);
  put64(b + 48, 9);
  floating(b + 56, 20);
  floating(b + 64, 30);
  floating(b + 72, .125);
  call(a, 12, b, 88, r);
  memset(b, 0, sizeof(b));
  context(a, b);
  put32(b + 24, 7);
  put64(b + 32, 7);
  call(a, 10, b, 40, r);
  memset(b, 0, sizeof(b));
  context(a, b);
  put32(b + 24, 2); /* high-level cycle also chooses Select */
  put32(b + 32, 2);
  call(a, 11, b, 40, r);
}
/* ANDROID-SCALE-001, UNDO-001/002, FFI-001: actual linked library and literal
 * packet. */
void check_scale(const char *fixture_path) {
  uint8_t p[72], r[512], b[64] = {0};
  FILE *fixture = fopen(fixture_path, "rb");
  assert(fixture);
  assert(fread(p, 1, sizeof(p), fixture) == sizeof(p));
  assert(fgetc(fixture) == EOF);
  assert(fclose(fixture) == 0);
  assert(get32(p + 8) == 11 && get32(p + 56) == 2);
  assert(number(p + 64) == 1.5);
  TestApp a = create();
  seed(a);
  bounds(a, 2);
  edit_tag(a, 8, r);
  context(a, b);
  put32(b + 24, 9);
  floating(b + 32, 4);
  floating(b + 40, -100);
  floating(b + 48, -200);
  call(a, 10, b, 56, r);
  uint64_t interaction = get64(r + 104), revision = get64(r + 88);
  context(a, p + 24); /* only opaque context changes in the literal request */
  NsV2Handle reply = 0;
  NsV2Error error;
  floating(p + 64, 0);
  assert(ns_v2_app_call(a.app, p, sizeof(p), &reply, &error) == 1);
  assert(!reply);
  put64(b, a.session);
  call(a, 3, b, 8, r);
  assert(get64(r + 104) == interaction && get64(r + 88) == revision);
  floating(p + 64, 1.5);
  assert(ns_v2_app_call(a.app, p, sizeof(p), &reply, &error) == 0);
  copy_reply(reply, r);
  assert(get64(r + 104) == 0);
  bounds(a, 3);
  revision = get64(r + 88);
  context(a, p + 24);
  floating(p + 64, 1);
  reply = 0;
  assert(ns_v2_app_call(a.app, p, sizeof(p), &reply, &error) == 0);
  copy_reply(reply, r);
  assert(get64(r + 88) == revision);
  context(a, p + 24);
  floating(p + 64, 1.5);
  reply = 0;
  assert(ns_v2_app_call(a.app, p, sizeof(p), &reply, &error) == 0);
  copy_reply(reply, r);
  bounds(a, 4.5);
  undo(a, r);
  bounds(a, 3);
  undo(a, r);
  bounds(a, 2);
  undo(a, r);
  assert(get64(r + 112) == 0);
  close_app(a);
  puts("PASS native top-left scale: literal packet, preview validation, "
       "relative repeat and undo");
}
