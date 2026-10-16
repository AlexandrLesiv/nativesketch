mod common;
use common::*;
use sketch_services::recovery::{RecoveryEnvelope, RecoveryError, RecoveryLimits, RecoveryStore};
use sketch_services::storage::NoFaults;
use std::fs;
#[test]
fn corrupt_latest_falls_back_without_deleting_older_valid_generation() {
    let temp = Temp::new();
    let store = RecoveryStore::open(temp.0.join("recovery"), RecoveryLimits::default()).unwrap();
    store.allocate(id(1)).unwrap();
    for generation in 1..=3 {
        store.publish(envelope(generation), &NoFaults).unwrap();
    }
    let root = temp.0.join("recovery").join(id(1).name());
    assert!(!root.join("0000000000000001.nskr").exists());
    fs::write(root.join("0000000000000003.nskr"), b"corrupt").unwrap();
    let candidate = store.load(id(1)).unwrap();
    assert_eq!(candidate.envelope.metadata().generation, 2);
    assert_eq!(candidate.rejected_generations, 1);
    assert!(root.join("0000000000000002.nskr").exists());
}
#[test]
fn identity_checksum_lengths_and_oversized_candidates_are_rejected() {
    let bytes = envelope(1).encode();
    for index in [0, 8, 24, 48, 56, bytes.len() - 1] {
        let mut corrupt = bytes.clone();
        corrupt[index] ^= 1;
        assert!(RecoveryEnvelope::decode(&corrupt).is_err());
    }
    for length in 0..bytes.len() {
        assert!(RecoveryEnvelope::decode(&bytes[..length]).is_err());
    }
    let temp = Temp::new();
    let store = RecoveryStore::open(temp.0.join("recovery"), RecoveryLimits::default()).unwrap();
    store.allocate(id(1)).unwrap();
    store.publish(envelope(1), &NoFaults).unwrap();
    let huge = temp
        .0
        .join("recovery")
        .join(id(1).name())
        .join("0000000000000002.nskr");
    fs::File::create(huge)
        .unwrap()
        .set_len(65 * 1024 * 1024)
        .unwrap();
    let candidate = store.load(id(1)).unwrap();
    assert_eq!(candidate.rejected_generations, 1);
    assert_eq!(candidate.envelope.metadata().generation, 1);
}
#[test]
fn quotas_preserve_unresolved_data_and_exclusive_id_allocation() {
    let temp = Temp::new();
    let size = envelope(1).encode().len() as u64;
    let store = RecoveryStore::open(
        temp.0.join("recovery"),
        RecoveryLimits {
            sessions: 1,
            bytes: size + 128,
        },
    )
    .unwrap();
    store.allocate(id(1)).unwrap();
    assert_eq!(store.allocate(id(1)), Err(RecoveryError::Capacity));
    store.publish(envelope(1), &NoFaults).unwrap();
    assert_eq!(
        store.publish(envelope(2), &NoFaults).unwrap_err(),
        RecoveryError::Capacity
    );
    assert_eq!(store.allocate(id(2)), Err(RecoveryError::Capacity));
    assert_eq!(store.load(id(1)).unwrap().envelope.metadata().generation, 1);
    assert_eq!(store.expire_resolved(u64::MAX).unwrap().removed, 0);
}
#[test]
fn recovered_document_is_new_dirty_unbound_and_readonly_cannot_restore() {
    use sketch_core::application::{Capabilities, EditorSession, ToolId};
    let original = session();
    let recovered = EditorSession::new_recovered(
        original.view_state().capabilities,
        ToolId::Pen,
        original.view_state().brush,
        envelope(1).into_document().unwrap(),
    )
    .unwrap();
    assert_ne!(
        original.view_state().session,
        recovered.view_state().session
    );
    assert!(recovered.view_state().document.dirty);
    assert_eq!(recovered.save_state().destination(), None);
    assert!(
        EditorSession::new_recovered(
            Capabilities {
                edit_document: false,
                export_document: true
            },
            ToolId::Select,
            original.view_state().brush,
            envelope(1).into_document().unwrap()
        )
        .is_err()
    );
}
#[cfg(unix)]
#[test]
fn recovery_symlink_entries_are_rejected_without_reading_or_deleting_target() {
    use std::os::unix::fs::symlink;
    let temp = Temp::new();
    let store = RecoveryStore::open(temp.0.join("recovery"), RecoveryLimits::default()).unwrap();
    store.allocate(id(1)).unwrap();
    let target = temp.0.join("outside");
    fs::write(&target, FIXTURE).unwrap();
    symlink(
        &target,
        temp.0
            .join("recovery")
            .join(id(1).name())
            .join("0000000000000001.nskr"),
    )
    .unwrap();
    assert!(store.load(id(1)).is_err());
    assert!(store.discard_step(id(1)).is_err());
    assert_eq!(fs::read(target).unwrap(), FIXTURE);
}
