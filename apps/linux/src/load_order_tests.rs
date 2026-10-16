use crate::{
    provider_work::{Job, execute},
    test_support::Fixture,
};
use sketch_app::{HostEffect, WorkItem};
use sketch_core::application::routing::*;
use std::path::PathBuf;
fn begin(f: &mut Fixture, path: PathBuf) -> WorkItem {
    f.file(FileIntent::Navigate(NavigationIntent::Open));
    if let Some(HostEffect::ConfirmUnsaved { token, .. }) = f.model.owner.effects.front().copied() {
        f.effect();
        f.reply(token, EffectDecision::Unsaved(UnsavedChoice::Discard));
    }
    f.choose(path);
    f.model.owner.app.take_work().unwrap().unwrap()
}
#[test]
fn older_read_finishing_after_newer_load_cannot_replace_new_document() {
    let mut f = Fixture::new();
    f.stroke();
    let older_path = f.directory.join("older.nsketch");
    std::fs::write(&older_path, f.snapshot()).unwrap();
    f.stroke();
    let newer_bytes = f.snapshot();
    let newer_path = f.directory.join("newer.nsketch");
    std::fs::write(&newer_path, &newer_bytes).unwrap();
    let older = begin(&mut f, older_path.clone());
    let older_info = older.info();
    f.model
        .owner
        .app
        .cancel_operation(older_info.operation)
        .unwrap();
    let newer = begin(&mut f, newer_path.clone());
    let newer_info = newer.info();
    let mut result = Some(execute(Job {
        work: newer,
        path: Some(newer_path),
    }));
    f.model.owner.app.complete_work(&mut result).unwrap();
    let outcome = f
        .model
        .owner
        .app
        .apply_ready(newer_info.session, newer_info.operation)
        .unwrap();
    f.model.owner.accept(outcome);
    let mut stale = Some(execute(Job {
        work: older,
        path: Some(older_path),
    }));
    f.model.owner.app.complete_work(&mut stale).unwrap();
    assert!(
        f.model
            .owner
            .app
            .apply_ready(older_info.session, older_info.operation)
            .is_err()
    );
    f.drain();
    assert_eq!(f.model.owner.state().count, 2);
    assert!(!f.model.owner.state().dirty);
    assert_eq!(f.snapshot(), newer_bytes);
}
