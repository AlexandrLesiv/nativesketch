mod common;
use common::*;
use sketch_services::recovery::{RecoveryEnvelope, RecoveryError, RecoveryLimits, RecoveryStore};
use sketch_services::storage::NoFaults;
use std::fs;
#[test]
fn resolution_reserves_aggregate_marker_staging_without_exceeding_budget() {
    let temp = Temp::new();
    let encoded_size = envelope(1).encode().len() as u64;
    let limit = encoded_size * 3 + 128;
    let root = temp.0.join("recovery");
    let store = RecoveryStore::open(
        root.clone(),
        RecoveryLimits {
            sessions: 3,
            bytes: limit,
        },
    )
    .unwrap();
    for number in 1..=3 {
        let mut meta = metadata(1);
        meta.identity = id(number);
        store.allocate(meta.identity).unwrap();
        store
            .publish(
                RecoveryEnvelope::new(meta, FIXTURE.to_vec()).unwrap(),
                &NoFaults,
            )
            .unwrap();
    }
    store.mark_resolved(id(1), 1, 1).unwrap();
    store.mark_resolved(id(2), 1, 1).unwrap();
    assert!(matches!(
        store.mark_resolved(id(3), 1, 1),
        Err(RecoveryError::Capacity)
    ));
    // Even replacement of an existing marker must reserve a temporary marker.
    assert!(matches!(
        store.mark_resolved(id(1), 1, 2),
        Err(RecoveryError::Capacity)
    ));
    let total: u64 = fs::read_dir(&root)
        .unwrap()
        .map(|entry| {
            fs::read_dir(entry.unwrap().path())
                .unwrap()
                .map(|file| file.unwrap().metadata().unwrap().len())
                .sum::<u64>()
        })
        .sum();
    assert_eq!(total, limit);
    assert_eq!(store.candidates().unwrap().candidates[0].identity, id(3));
}
