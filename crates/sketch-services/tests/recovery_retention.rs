mod common;
use common::*;
use sketch_services::recovery::{DiscardProgress, RecoveryLimits, RecoveryStore};
use sketch_services::storage::NoFaults;
use std::fs;
#[test]
fn resolved_candidate_is_hidden_and_newer_generation_remains_unresolved() {
    let temp = Temp::new();
    let store = RecoveryStore::open(temp.0.join("recovery"), RecoveryLimits::default()).unwrap();
    store.allocate(id(1)).unwrap();
    store.publish(envelope(1), &NoFaults).unwrap();
    store.mark_resolved(id(1), 1, 10).unwrap();
    assert!(store.candidates().unwrap().candidates.is_empty());
    store.publish(envelope(2), &NoFaults).unwrap();
    assert_eq!(store.candidates().unwrap().candidates[0].generation, 2);
    assert_eq!(store.expire_resolved(u64::MAX).unwrap().removed, 0);
    store.mark_resolved(id(1), 2, 20).unwrap();
    assert_eq!(store.expire_resolved(20).unwrap().removed, 0);
    assert_eq!(
        store
            .expire_resolved(20 + 7 * 24 * 60 * 60)
            .unwrap()
            .removed,
        1
    );
}
#[test]
fn corrupt_or_foreign_resolution_marker_cannot_authorize_deletion() {
    let temp = Temp::new();
    let store = RecoveryStore::open(temp.0.join("recovery"), RecoveryLimits::default()).unwrap();
    store.allocate(id(1)).unwrap();
    store.publish(envelope(1), &NoFaults).unwrap();
    store.mark_resolved(id(1), 1, 1).unwrap();
    let path = temp.0.join("recovery").join(id(1).name()).join("resolved");
    let valid = fs::read(&path).unwrap();
    store.publish(envelope(2), &NoFaults).unwrap();
    for index in [0, 8, 40, 48, 63] {
        let mut corrupt = valid.clone();
        corrupt[index] ^= 2;
        fs::write(&path, corrupt).unwrap();
        assert_eq!(store.expire_resolved(u64::MAX).unwrap().rejected_markers, 1);
        assert_eq!(store.load(id(1)).unwrap().envelope.metadata().generation, 2);
        assert_eq!(store.candidates().unwrap().invalid_resolution_markers, 1);
    }
}
#[test]
fn overfull_session_does_not_block_other_recovery_or_bounded_management() {
    let temp = Temp::new();
    let store = RecoveryStore::open(temp.0.join("recovery"), RecoveryLimits::default()).unwrap();
    store.allocate(id(1)).unwrap();
    store.publish(envelope(1), &NoFaults).unwrap();
    store.allocate(id(2)).unwrap();
    let mut meta = metadata(1);
    meta.identity = id(2);
    store
        .publish(
            sketch_services::recovery::RecoveryEnvelope::new(meta, FIXTURE.to_vec()).unwrap(),
            &NoFaults,
        )
        .unwrap();
    for index in 0..64 {
        fs::write(
            temp.0
                .join("recovery")
                .join(id(1).name())
                .join(format!(".native-sketch-dead-{index}.tmp")),
            b"stage",
        )
        .unwrap();
    }
    assert!(store.load(id(1)).is_err());
    assert_eq!(
        store.load(id(2)).unwrap().envelope.metadata().identity,
        id(2)
    );
    let listing = store.candidates().unwrap();
    assert_eq!(listing.rejected_sessions, 1);
    assert_eq!(listing.candidates.len(), 1);
    assert!(store.cleanup_staging_step(id(1)).unwrap() <= 32);
    assert!(store.load(id(1)).is_ok());
    assert_eq!(
        store.discard_step(id(1)).unwrap(),
        DiscardProgress::MoreEntries
    );
    assert_eq!(
        store.discard_step(id(1)).unwrap(),
        DiscardProgress::Complete
    );
    assert!(store.load(id(2)).is_ok());
}
#[test]
fn corrupt_newest_does_not_hide_valid_unresolved_fallback_after_older_resolution() {
    let temp = Temp::new();
    let store = RecoveryStore::open(temp.0.join("recovery"), RecoveryLimits::default()).unwrap();
    store.allocate(id(1)).unwrap();
    store.publish(envelope(1), &NoFaults).unwrap();
    store.mark_resolved(id(1), 1, 1).unwrap();
    store.publish(envelope(2), &NoFaults).unwrap();
    store.publish(envelope(3), &NoFaults).unwrap();
    fs::write(
        temp.0
            .join("recovery")
            .join(id(1).name())
            .join("0000000000000003.nskr"),
        b"corrupt",
    )
    .unwrap();
    let listing = store.candidates().unwrap();
    assert_eq!(listing.candidates[0].generation, 2);
    assert_eq!(listing.rejected_generations, 1);
    assert_eq!(store.expire_resolved(u64::MAX).unwrap().removed, 0);
}
