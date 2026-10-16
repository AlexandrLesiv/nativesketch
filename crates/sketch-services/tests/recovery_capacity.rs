mod common;
use common::*;
use sketch_services::recovery::{
    DiscardProgress, RecoveryEnvelope, RecoveryError, RecoveryLimits, RecoveryStore,
};
use sketch_services::storage::NoFaults;
use std::fs;

#[test]
fn inventory_reports_real_capacity_and_reclaims_it_after_selected_discard() {
    let temp = Temp::new();
    let store = RecoveryStore::open(temp.0.join("recovery"), RecoveryLimits::default()).unwrap();
    let empty = store.candidates().unwrap();
    assert_eq!(empty.usage.unwrap().bytes, 0);
    assert!(!empty.capacity_blocked);
    let mut bytes = 0;
    for number in 1..=20 {
        let mut meta = metadata(1);
        meta.identity = id(number);
        store.allocate(meta.identity).unwrap();
        let envelope = RecoveryEnvelope::new(meta, FIXTURE.to_vec()).unwrap();
        bytes += envelope.encode().len() as u64;
        store.publish(envelope, &NoFaults).unwrap();
    }
    let full = store.candidates().unwrap();
    assert_eq!(full.candidates.len(), 20);
    assert_eq!(full.usage.unwrap().sessions, 20);
    assert_eq!(full.usage.unwrap().bytes, bytes);
    assert_eq!(full.limit_bytes, RecoveryLimits::default().bytes);
    assert!(full.capacity_blocked);
    assert!(matches!(
        store.allocate(id(21)),
        Err(RecoveryError::Capacity)
    ));
    assert_eq!(store.candidates().unwrap().usage, full.usage);
    assert_eq!(
        store.discard_step(id(1)).unwrap(),
        DiscardProgress::Complete
    );
    let freed = store.candidates().unwrap();
    assert_eq!(freed.candidates.len(), 19);
    assert!(!freed.capacity_blocked);
    assert!(freed.usage.unwrap().bytes < bytes);
    store.allocate(id(21)).unwrap();
    // An empty allocated directory consumes a real slot, even without a visible candidate.
    let allocated = store.candidates().unwrap();
    assert_eq!(allocated.candidates.len(), 19);
    assert!(allocated.capacity_blocked);
    assert_eq!(allocated.usage.unwrap().sessions, 20);
}

#[test]
fn resolved_entries_and_marker_budget_are_counted_without_hiding_unknown_usage() {
    let temp = Temp::new();
    let limit = envelope(1).encode().len() as u64 + 128;
    let root = temp.0.join("recovery");
    let store = RecoveryStore::open(
        root.clone(),
        RecoveryLimits {
            sessions: 2,
            bytes: limit,
        },
    )
    .unwrap();
    store.allocate(id(1)).unwrap();
    store.publish(envelope(1), &NoFaults).unwrap();
    assert!(store.candidates().unwrap().capacity_blocked);
    assert!(matches!(
        store.publish(envelope(2), &NoFaults),
        Err(RecoveryError::Capacity)
    ));
    store.mark_resolved(id(1), 1, 1).unwrap();
    let resolved = store.candidates().unwrap();
    assert!(resolved.candidates.is_empty());
    assert_eq!(resolved.usage.unwrap().sessions, 1);
    assert!(resolved.usage.unwrap().bytes > envelope(1).encode().len() as u64);
    fs::write(root.join("foreign-entry"), b"not ours").unwrap();
    let unsafe_listing = store.candidates().unwrap();
    assert!(unsafe_listing.usage.is_none());
    assert_eq!(unsafe_listing.rejected_sessions, 1);
    assert_eq!(fs::read(root.join("foreign-entry")).unwrap(), b"not ours");
}
