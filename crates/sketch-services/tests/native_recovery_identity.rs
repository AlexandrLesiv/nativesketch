#[path = "native_runtime/recovery_support.rs"]
mod recovery_support;
#[path = "native_runtime/support.rs"]
mod support;
use recovery_support::*;
use sketch_core::application::{routing::*, *};
use sketch_services::{native_runtime::IntentEffect, recovery::*};
use std::{fs, time::Duration};
use support::*;
#[test]
fn cancelled_recovery_must_not_follow_later_new_document() {
    let (mut r, w, root, store, id) = configured();
    r.list_recoveries(1).unwrap();
    let ev = ready(&mut r);
    let item = r.inventory(ev.id).unwrap().items[0].0;
    begin(&mut r, w);
    edit(&mut r, w, EditorCommand::CommitStroke(PointerId(7)));
    let result = r.open_recovery(w, ctx(&r, w), item).unwrap();
    let IntentEffect::Unsaved(token) = result.effect else {
        panic!()
    };
    r.reply(w, ctx(&r, w), token, EffectDecision::Cancel)
        .unwrap();
    r.release_event(ev.id).unwrap();
    let IntentEffect::Unsaved(token) =
        file(&mut r, w, FileIntent::Navigate(NavigationIntent::New)).effect
    else {
        panic!()
    };
    r.reply(
        w,
        ctx(&r, w),
        token,
        EffectDecision::Unsaved(UnsavedChoice::Discard),
    )
    .unwrap();
    let ev = ready(&mut r);
    r.apply_event(w, ev.id).unwrap();
    assert!(r.session(w).unwrap().recovered_from.is_none());
    save_as(&mut r, w, root.join("unrelated.nsketch"));
    r.list_recoveries(3).unwrap();
    let ev = ready(&mut r);
    r.release_event(ev.id).unwrap();
    assert_eq!(
        store.candidates().unwrap().candidates.len(),
        1,
        "unopened recovery hidden as resolved"
    );
    assert!(store.load(id).is_ok());
    close(r, w);
    fs::remove_dir_all(root).unwrap();
}
#[test]
fn replacing_session_clears_checkpoint_revision_knowledge() {
    let (mut r, w, root, store, _) = configured();
    let identity = RecoveryId::from_bytes([89; 16]).unwrap();
    let session = r.session(w).unwrap().app.projection().editor.session.get();
    r.register_recovery(w, session, identity, "previous".into())
        .unwrap();
    let e = ready(&mut r);
    r.release_event(e.id).unwrap();
    begin(&mut r, w);
    edit(&mut r, w, EditorCommand::CommitStroke(PointerId(7)));
    std::thread::sleep(Duration::from_millis(2050));
    r.idle().unwrap();
    let e = ready(&mut r);
    r.apply_event(w, e.id).unwrap();
    assert!(r.session(w).unwrap().last_checkpoint.is_some());
    let IntentEffect::Unsaved(token) =
        file(&mut r, w, FileIntent::Navigate(NavigationIntent::New)).effect
    else {
        panic!("unsaved");
    };
    r.reply(
        w,
        ctx(&r, w),
        token,
        EffectDecision::Unsaved(UnsavedChoice::Discard),
    )
    .unwrap();
    let e = ready(&mut r);
    r.apply_event(w, e.id).unwrap();
    assert!(r.session(w).unwrap().last_checkpoint.is_none());
    begin(&mut r, w);
    edit(&mut r, w, EditorCommand::CommitStroke(PointerId(7)));
    save_as(&mut r, w, root.join("different-session.nsketch"));
    r.list_recoveries(3).unwrap();
    let e = ready(&mut r);
    r.release_event(e.id).unwrap();
    assert!(
        store
            .candidates()
            .unwrap()
            .candidates
            .iter()
            .any(|c| c.identity == identity)
    );
    close(r, w);
    fs::remove_dir_all(root).unwrap();
}
