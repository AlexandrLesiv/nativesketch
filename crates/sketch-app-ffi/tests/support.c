#include "support.h"

void put32(uint8_t *p, uint32_t v) {
    for (size_t i = 0; i < 4; i++) p[i] = (uint8_t)(v >> (8 * i));
}
void put64(uint8_t *p, uint64_t v) {
    for (size_t i = 0; i < 8; i++) p[i] = (uint8_t)(v >> (8 * i));
}
uint32_t get32(const uint8_t *p) {
    uint32_t v = 0;
    for (size_t i = 0; i < 4; i++) v |= (uint32_t)p[i] << (8 * i);
    return v;
}
uint64_t get64(const uint8_t *p) {
    uint64_t v = 0;
    for (size_t i = 0; i < 8; i++) v |= (uint64_t)p[i] << (8 * i);
    return v;
}
size_t frame(uint8_t *p, uint32_t op, const uint8_t *body, size_t n) {
    memset(p, 0, 24 + n);
    memcpy(p, "NSAP", 4);
    p[4] = 2;
    put32(p + 8, op);
    put32(p + 12, (uint32_t)n);
    put64(p + 16, UINT64_C(0xfedcba9876543210));
    if (n) memcpy(p + 24, body, n);
    return 24 + n;
}
size_t copy_reply(NsV2Handle reply, uint8_t *out) {
    NsV2Error e;
    size_t n = 0, w = 0;
    assert(ns_v2_buffer_len(reply, &n, &e) == 0);
    assert(n <= 512);
    assert(ns_v2_buffer_copy(reply, 0, out, n, &w, &e) == 0);
    assert(w == n);
    assert(ns_v2_buffer_free(&reply, &e) == 0);
    assert(reply == 0);
    return n;
}
void call(TestApp a, uint32_t op, const uint8_t *body, size_t n, uint8_t *out) {
    uint8_t p[256];
    NsV2Error e;
    NsV2Handle r = 0;
    size_t len = frame(p, op, body, n);
    assert(ns_v2_app_call(a.app, p, len, &r, &e) == 0);
    copy_reply(r, out);
}
static TestApp configured(uint32_t opcode, uint32_t policy) {
    uint8_t b[64], p[128], r[512];
    NsV2Error e;
    TestApp a = {0, 0};
    uint64_t limits[] = {16, 32, 64, 32, 134217728, 67108864, 16, 134217728};
    for (size_t i = 0; i < 8; i++) put64(b + 8 * i, limits[i]);
    size_t n = frame(p, 0, b, 64);
    assert(ns_v2_app_new(p, n, &a.app, &e) == 0);
    memset(b, 0, sizeof(b));
    put32(b, 3);
    put32(b + 4, 1);
    double brush[] = {0.25, 0.5, 0.75, 1.0, 2.0};
    for (size_t i = 0; i < 5; i++) {
        uint64_t bits;
        memcpy(&bits, &brush[i], 8);
        put64(b + 8 + 8 * i, bits);
    }
    put32(b + 48, policy);
    call(a, opcode, b, opcode == 8 ? 56 : 48, r);
    a.session = get64(r + 72);
    return a;
}
void context(TestApp a, uint8_t *b) {
    uint8_t request[8], r[512];
    put64(request, a.session);
    call(a, 3, request, 8, r);
    put64(b, a.session);
    put64(b + 8, get64(r + 80));
    put64(b + 16, get64(r + 104));
}
uint64_t queue_save(TestApp a) {
    uint8_t b[64] = {0}, r[512];
    context(a, b);
    put32(b + 24, 2);
    call(a, 20, b, 32, r);
    uint64_t effect = get64(r + 240);
    assert(get32(r + 232) == 1);
    context(a, b);
    put64(b + 24, effect);
    put32(b + 32, 1);
    put32(b + 36, 7);
    put64(b + 40, UINT64_C(0xfedcba9876543211));
    put64(b + 48, 1);
    put64(b + 56, 1);
    call(a, 22, b, 64, r);
    assert(get32(r + 232) == 5);
    return get64(r + 248);
}
void close_app(TestApp a) {
    uint8_t b[64] = {0}, r[512], p[128];
    NsV2Error e;
    context(a, b);
    put32(b + 24, 6);
    call(a, 20, b, 32, r);
    assert(get32(r + 232) == 6);
    uint64_t effect = get64(r + 240);
    context(a, b);
    put64(b + 24, effect);
    put32(b + 32, 5);
    put32(b + 36, 0);
    call(a, 21, b, 40, r);
    assert(get32(r + 232) == 7);
    put64(b, a.session);
    call(a, 2, b, 8, r);
    for (;;) {
        NsV2Handle w = 0, reply = 0, result = 0, input = 0;
        size_t n = frame(p, 30, NULL, 0);
        assert(ns_v2_app_take_work(a.app, p, n, &w, &reply, &e) == 0);
        copy_reply(reply, r);
        reply = 0;
        if (!w) break;
        n = frame(p, 106, NULL, 0);
        assert(ns_v2_work_call(&w, p, n, &input, &result, &reply, &e) == 0);
        assert(!w);
        copy_reply(reply, r);
        reply = 0;
        n = frame(p, 31, NULL, 0);
        assert(ns_v2_app_complete(a.app, p, n, &result, &reply, &e) == 0);
        copy_reply(reply, r);
    }
    do { call(a, 34, NULL, 0, r); } while (get32(r + 72));
    assert(ns_v2_app_free(&a.app, &e) == 0);
    assert(a.app == 0);
}

TestApp create(void) { return configured(1, 0); }
TestApp create_policy(uint32_t policy) { return configured(8, policy); }
