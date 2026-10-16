#include "support.h"

int main(void) {
    _Static_assert(sizeof(NsV2Error) == 72, "fixed allocation-free error record");
    TestApp a = create();
    uint8_t p[128], bytes[512];
    NsV2Error e;
    NsV2Handle r = 71;
    size_t n = frame(p, 5, NULL, 0);
    assert(ns_v2_app_call(a.app, p, n, &r, &e) == 1);
    assert(r == 71);
    r = 0;
    p[n++] = 42;
    assert(ns_v2_app_call(a.app, p, n, &r, &e) == 1);
    assert(r == 0);
    n = frame(p, 5, NULL, 0);
    assert(ns_v2_app_call(a.app, p, n, &r, &e) == 0);
    for (size_t i = 0; i < 72; i++) assert(e.bytes[i] == 0);
    assert(ns_v2_input_free(&r, &e) == 1);
    assert(r != 0);
    NsV2Handle old = r;
    size_t count = 0;
    assert(ns_v2_buffer_len(r, &count, &e) == 0);
    size_t written = 0;
    memset(bytes, 91, sizeof(bytes));
    assert(ns_v2_buffer_copy(r, count + 1, bytes, 8, &written, &e) == 1);
    assert(!written && bytes[0] == 91);
    assert(ns_v2_buffer_copy(r, count, NULL, 0, &written, &e) == 0);
    assert(!written);
    assert(ns_v2_buffer_free(&r, &e) == 0 && r == 0);
    assert(ns_v2_buffer_free(&old, &e) == 7 && old != 0);
    assert(ns_v2_app_call(a.app, NULL, 1, &r, &e) == 1);
    assert(r == 0);
    close_app(a);
    check_worker_roundtrip();
    puts("PASS native v2 C consumer: transfer, pthread worker, rejection, shutdown");
    return 0;
}
