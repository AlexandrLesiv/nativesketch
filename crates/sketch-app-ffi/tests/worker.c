#include "support.h"

typedef struct { TestApp owner; NsV2Handle work; NsV2Handle result; } Transfer;
static void *run_worker(void *argument) {
    Transfer *t = argument;
    uint8_t p[128], r[512], body[8] = {4};
    NsV2Error e;
    NsV2Handle reply = 0, input = 0, result = 0;
    size_t n = frame(p, 5, NULL, 0);
    assert(ns_v2_app_call(t->owner.app, p, n, &reply, &e) == 11);
    assert(reply == 0);
    n = frame(p, 101, NULL, 0);
    reply = 91;
    NsV2Handle original = t->work;
    assert(ns_v2_work_call(&t->work, p, n, &input, &result, &reply, &e) == 1);
    assert(reply == 91 && t->work == original && result == 0);
    reply = 0;
    assert(ns_v2_work_call(&t->work, p, n, &input, &result, &reply, &e) == 0);
    assert(t->work && result == 0);
    copy_reply(reply, r);
    reply = 0;
    original = t->work;
    n = frame(p, 104, body, 8);
    assert(ns_v2_work_call(&t->work, p, n, &input, &result, &reply, &e) == 8);
    assert(t->work == original && result == 0 && reply == 0);
    assert(get32(e.bytes + 24) == 8);
    body[0] = 1;
    body[4] = 3;
    n = frame(p, 104, body, 8);
    assert(ns_v2_work_call(&t->work, p, n, &input, &result, &reply, &e) == 0);
    assert(t->work == 0 && result != 0);
    copy_reply(reply, r);
    t->result = result;
    return NULL;
}
void check_worker_roundtrip(void) {
    TestApp a = create_policy(1);
    check_bounds(a);
    uint64_t op = queue_save(a);
    uint8_t p[128], r[512], body[16];
    NsV2Error e;
    NsV2Handle w = 0, reply = 0;
    size_t n = frame(p, 30, NULL, 0);
    assert(ns_v2_app_take_work(a.app, p, n, &w, &reply, &e) == 0);
    copy_reply(reply, r);
    Transfer transfer = {a, w, 0};
    pthread_t thread;
    assert(pthread_create(&thread, NULL, run_worker, &transfer) == 0);
    assert(pthread_join(thread, NULL) == 0);
    assert(transfer.result != 0);
    assert(ns_v2_result_retire_stopped(&transfer.result, &e) == 9);
    TestApp other = create();
    reply = 0;
    n = frame(p, 31, NULL, 0);
    NsV2Handle original = transfer.result;
    assert(ns_v2_app_complete(other.app, p, n, &transfer.result, &reply, &e) == 11);
    assert(transfer.result == original && reply == 0);
    close_app(other);
    assert(ns_v2_app_complete(a.app, p, n, &transfer.result, &reply, &e) == 0);
    assert(transfer.result == 0);
    copy_reply(reply, r);
    put64(body, a.session);
    put64(body + 8, op);
    call(a, 32, body, 16, r);
    assert((get32(r + 156) & 1) == 0);
    check_bound_chooser(a);
    close_app(a);
}
