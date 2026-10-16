#[path = "native_runtime/recovery_support.rs"]
mod recovery_support;
#[path = "native_runtime/support.rs"]
mod support;
use recovery_support::*;
use sketch_core::application::*;
use sketch_services::{recovery::*, storage::NoFaults};
use std::{fs, time::Duration};
use support::*;
#[test]
fn first_checkpoint_allocation_failure_retries_after_capacity_is_released() {
    let (mut r, w, root, store, _) = configured();
    let identity = RecoveryId::from_bytes([88; 16]).unwrap();
    let session = r.session(w).unwrap().app.projection().editor.session.get();
    r.register_recovery(w, session, identity, "current".into())
        .unwrap();
    let e = ready(&mut r);
    r.release_event(e.id).unwrap();
    let QueryResult::Document(snapshot) = r
        .session(w)
        .unwrap()
        .app
        .query(EditorQuery::CommittedSnapshot)
        .unwrap()
    else {
        panic!()
    };
    let bytes = snapshot.encode().unwrap();
    for n in 1..20 {
        let identity = RecoveryId::from_bytes([n; 16]).unwrap();
        store.allocate(identity).unwrap();
        store
            .publish(
                RecoveryEnvelope::new(
                    CheckpointMetadata {
                        identity,
                        generation: 1,
                        revision: 0,
                        created_seconds: 1,
                        display_name: "fixture".into(),
                    },
                    bytes.clone(),
                )
                .unwrap(),
                &NoFaults,
            )
            .unwrap();
    }
    begin(&mut r, w);
    edit(&mut r, w, EditorCommand::CommitStroke(PointerId(7)));
    std::thread::sleep(Duration::from_millis(2050));
    r.idle().unwrap();
    let e = ready(&mut r);
    let a = r.apply_event(w, e.id).unwrap();
    assert!(matches!(
        a.outcome.save.unwrap().result,
        sketch_core::application::storage::SaveResult::Failed(_)
    ));
    store
        .discard_step(RecoveryId::from_bytes([1; 16]).unwrap())
        .unwrap();
    r.retry_recovery(2, w).unwrap();
    let e = ready(&mut r);
    r.release_event(e.id).unwrap();
    r.idle().unwrap();
    let e = ready(&mut r);
    let a = r.apply_event(w, e.id).unwrap();
    assert!(
        matches!(
            a.outcome.save.unwrap().result,
            sketch_core::application::storage::SaveResult::Written(_)
        ),
        "retry failed after capacity freed"
    );
    assert!(store.load(identity).is_ok());
    save_as(&mut r, w, root.join("saved.nsketch"));
    let event = ready(&mut r);
    r.release_event(event.id).unwrap();
    close(r, w);
    fs::remove_dir_all(root).unwrap();
}
#[test]
fn rejected_identity_cannot_be_authorized_by_window_or_global_retry() {
    let (mut r, w, root, store, id) = configured();
    let session = r.session(w).unwrap().app.projection().editor.session.get();
    r.register_recovery(w, session, id, "different document".into())
        .unwrap();
    let e = ready(&mut r);
    assert!(e.failure.is_some());
    r.release_event(e.id).unwrap();
    begin(&mut r, w);
    edit(&mut r, w, EditorCommand::CommitStroke(PointerId(7)));
    std::thread::sleep(Duration::from_millis(2050));
    for window in [0, w, w] {
        r.retry_recovery(29, window).unwrap();
        let e = ready(&mut r);
        r.release_event(e.id).unwrap();
        assert!(!r.session(w).unwrap().recovery.as_ref().unwrap().approved);
        r.idle().unwrap();
        assert_eq!(store.load(id).unwrap().envelope.metadata().generation, 1);
        assert_eq!(r.session(w).unwrap().app.projection().outstanding_saves, 0);
    }
    save_as(&mut r, w, root.join("unrelated.nsketch"));
    close(r, w);
    fs::remove_dir_all(root).unwrap();
}
