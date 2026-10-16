use super::*;
#[test]
fn reply_exhaustion_preserves_issued_work_and_recovers_after_release() {
    let _serial = serial();
    let a = app();
    let s = session(a);
    let op = queued(a, s, 2);
    let w = take(a);
    let replies: Vec<_> = (0..64)
        .map(|_| app_call(a, &packet(5, &[])).unwrap())
        .collect();
    assert_eq!(work_call(w, &packet(101, &[]), 0).unwrap_err().status, 3);
    for r in replies {
        buffer_free(r).unwrap();
    }
    complete(a, finish(w));
    call(a, &packet(32, &words(&[s, op])));
    close(a, s);
}
#[test]
fn registry_reservation_exhaustion_precedes_mutation_and_cleanup_needs_no_slot() {
    use crate::{objects::Kind, registry::Slot};
    let _serial = serial();
    let a = app();
    let mut slots = Vec::new();
    while let Ok(s) = Slot::reserve(Kind::Buffer, None) {
        slots.push(s);
    }
    assert_eq!(slots.len(), 4095);
    assert_eq!(app_call(a, &fixture("create.cap3")).unwrap_err().status, 3);
    drop(slots);
    let bytes = call(a, &packet(5, &[]));
    assert_eq!(u64_at(&bytes, 72), 0);
    app_free(a).unwrap();
}
#[test]
fn uploads_have_count_and_byte_bounds_and_never_reuse_handles() {
    let _serial = serial();
    let ids: Vec<_> = (0..64).map(|_| upload_new(0).unwrap()).collect();
    assert_eq!(upload_new(0).unwrap_err().status, 3);
    let last = *ids.last().unwrap();
    for id in ids {
        upload_free(id).unwrap();
    }
    let id = upload_new(4).unwrap();
    assert!(id > last);
    assert_eq!(upload_len(id).unwrap(), 4);
    upload_free(id).unwrap();
    assert_eq!(upload_free(id).unwrap_err().status, 7);
    assert_eq!(upload_new(64 * 1024 * 1024 + 2).unwrap_err().status, 3);
}
#[test]
fn encode_capacity_failure_returns_real_result_and_consumes_work() {
    let _serial = serial();
    let mut config = fixture("bootstrap");
    config[64..72].copy_from_slice(&24u64.to_le_bytes());
    let a = app_new(&config).unwrap();
    let s = session(a);
    draw(a, s);
    let op = queued(a, s, 2);
    let w = take(a);
    let o = work_call(w, &packet(101, &[]), 0).unwrap();
    assert!(o.work_consumed);
    assert_ne!(o.result, 0);
    assert_eq!(u32_at(&copy_free(o.reply), 72), 1);
    assert_eq!(work_call(w, &packet(100, &[]), 0).unwrap_err().status, 7);
    complete(a, o.result);
    let r = call(a, &packet(32, &words(&[s, op])));
    assert_eq!(u32_at(&r, 156) & 1, 1);
    close(a, s);
}

#[test]
fn completion_slot_exhaustion_retains_actual_result_for_later_owner_apply() {
    use crate::{objects::Kind, registry::Slot};
    let _serial = serial();
    let a = app();
    let s = session(a);
    let op = queued(a, s, 2);
    let r = finish(take(a));
    let mut slots = Vec::new();
    while let Ok(s) = Slot::reserve(Kind::Buffer, None) {
        slots.push(s);
    }
    assert_eq!(app_complete(a, &packet(31, &[]), r).unwrap_err().status, 3);
    drop(slots);
    complete(a, r);
    call(a, &packet(32, &words(&[s, op])));
    close(a, s);
}
