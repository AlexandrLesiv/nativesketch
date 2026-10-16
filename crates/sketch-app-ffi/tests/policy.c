#include "support.h"
static void put_double(uint8_t *p, double value) {
    uint64_t bits;
    memcpy(&bits, &value, 8);
    put64(p, bits);
}
static double get_double(const uint8_t *p) {
    uint64_t bits = get64(p);
    double value;
    memcpy(&value, &bits, 8);
    return value;
}
void check_bounds(TestApp a) {
    uint8_t body[96] = {0}, r[512];
    put64(body, a.session);
    call(a, 9, body, 8, r);
    assert(get32(r + 8) == UINT32_C(0x80000009));
    assert(get64(r + 16) == UINT64_C(0xfedcba9876543210));
    assert(get64(r + 80) == 0 && get32(r + 88) == 0);
    for (size_t i = 92; i < 128; i++) assert(r[i] == 0);
    uint64_t document = get64(r + 72);
    context(a, body);
    put32(body + 24, 6);
    put64(body + 32, 7);
    put32(body + 40, 2);
    call(a, 10, body, 48, r);
    memset(body, 0, sizeof(body));
    context(a, body);
    put64(body + 24, 7);
    put32(body + 32, 1);
    put64(body + 48, 9);
    put_double(body + 56, 900e6 + 0.125);
    put_double(body + 64, -700e6 - 0.375);
    put_double(body + 72, 0.125);
    call(a, 12, body, 88, r);
    memset(body, 0, sizeof(body));
    context(a, body);
    put32(body + 24, 7);
    put64(body + 32, 7);
    call(a, 10, body, 40, r);
    put64(body, a.session);
    call(a, 9, body, 8, r);
    assert(get64(r + 72) == document && get64(r + 80) == 1);
    assert(get32(r + 88) == 1 && get32(r + 92) == 0);
    assert(get_double(r + 96) == 900e6 - 0.875);
    assert(get_double(r + 104) == -700e6 - 1.375);
    assert(get_double(r + 112) == 2.0 && get_double(r + 120) == 2.0);
}
void check_bound_chooser(TestApp a) {
    uint8_t body[64] = {0}, r[512];
    context(a, body);
    put32(body + 24, 1);
    call(a, 20, body, 32, r);
    assert(get32(r + 232) == 1 && get32(r + 236) == 2);
    uint64_t effect = get64(r + 240);
    context(a, body);
    put64(body + 24, effect);
    put32(body + 32, 1);
    call(a, 21, body, 40, r);
    assert(get32(r + 232) == 8);
}
