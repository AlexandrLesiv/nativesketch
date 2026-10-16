#[path = "native_runtime/recovery_support.rs"]
mod recovery_support;
#[path = "native_runtime/support.rs"]
mod support;
use recovery_support::*;
use sketch_core::application::{routing::*, *};
use sketch_services::{native_runtime::IntentEffect, recovery::RecoveryId};
use support::*;
#[test]
fn stale_registration_success_or_failure_cannot_change_new_or_open_replacement() {
    for (open, succeeds) in [(false, true), (false, false), (true, true), (true, false)] {
        let (mut r, w, root, store, id) = configured();
        let old_session = r.session(w).unwrap().app.projection().editor.session.get();
        r.register_recovery(w, old_session, id, "old".into())
            .unwrap();
        let e = ready(&mut r);
        assert!(e.failure.is_some());
        r.release_event(e.id).unwrap();
        if open {
            let path = root.join("open.nsketch");
            let QueryResult::Document(snapshot) = r
                .session(w)
                .unwrap()
                .app
                .query(EditorQuery::CommittedSnapshot)
                .unwrap()
            else {
                panic!("snapshot")
            };
            std::fs::write(&path, snapshot.encode().unwrap()).unwrap();
            let IntentEffect::Source(token) =
                file(&mut r, w, FileIntent::Navigate(NavigationIntent::Open)).effect
            else {
                panic!("source")
            };
            r.supply_location(w, ctx(&r, w), token, 19, path).unwrap();
            let e = ready(&mut r);
            r.apply_event(w, e.id).unwrap();
        } else {
            file(&mut r, w, FileIntent::Navigate(NavigationIntent::New));
        }
        let new_ready = ready(&mut r);
        if succeeds {
            store.discard_step(id).unwrap();
        }
        r.retry_recovery(123, w).unwrap();
        r.apply_event(w, new_ready.id).unwrap();
        let session = r.session(w).unwrap().app.projection().editor.session.get();
        assert_ne!(session, old_session);
        r.register_recovery(
            w,
            session,
            RecoveryId::from_bytes([99; 16]).unwrap(),
            "new".into(),
        )
        .unwrap();
        let old_retry = ready(&mut r);
        assert_eq!(old_retry.request, 123);
        assert_eq!(old_retry.operation_kind, 205);
        assert_eq!(old_retry.session, old_session);
        assert_eq!(old_retry.failure.is_none(), succeeds);
        let registration = r.session(w).unwrap().recovery.as_ref().unwrap();
        assert!(!registration.approved);
        assert_eq!(
            registration.state, 1,
            "old completion changed new registration"
        );
        r.release_event(old_retry.id).unwrap();
        let new_registration = ready(&mut r);
        r.release_event(new_registration.id).unwrap();
        assert!(r.session(w).unwrap().recovery.as_ref().unwrap().approved);
        begin(&mut r, w);
        edit(&mut r, w, EditorCommand::CancelInteraction);
        save_as(&mut r, w, root.join("final.nsketch"));
        close(r, w);
        std::fs::remove_dir_all(root).unwrap();
    }
}
