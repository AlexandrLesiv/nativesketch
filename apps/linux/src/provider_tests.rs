use crate::{
    provider_work::{Job, execute},
    test_support::Fixture,
};
use sketch_app::{HostEffect, WorkKind};
use sketch_core::application::routing::*;
#[test]
fn delayed_save_keeps_newer_edits_dirty_and_release_is_once() {
    let mut f = Fixture::new();
    f.stroke();
    f.file(FileIntent::SaveAs);
    let path = f.directory.join("delayed.nsketch");
    f.choose(path.clone());
    let work = f.model.owner.app.take_work().unwrap().unwrap();
    assert_eq!(work.info().kind, WorkKind::Write);
    f.stroke();
    let mut result = Some(execute(Job {
        work,
        path: Some(path.clone()),
    }));
    let operation = result.as_ref().unwrap().info().operation;
    f.model.owner.app.complete_work(&mut result).unwrap();
    assert!(result.is_none());
    let outcome = f
        .model
        .owner
        .app
        .apply_ready(f.model.owner.session, operation)
        .unwrap();
    f.model.owner.accept(outcome);
    assert!(f.model.owner.state().dirty);
    assert_eq!(
        u64::from_le_bytes(std::fs::read(path).unwrap()[8..16].try_into().unwrap()),
        1
    );
    assert_eq!(f.model.owner.state().count, 2);
}
#[test]
fn stale_chooser_cannot_consume_current_navigation() {
    let mut f = Fixture::new();
    f.file(FileIntent::Navigate(NavigationIntent::Open));
    let HostEffect::ChooseSource(old) = f.effect() else {
        panic!()
    };
    let context = f.model.owner.context().unwrap();
    f.reply(old, EffectDecision::Cancel);
    assert!(matches!(f.effect(), HostEffect::Cancelled));
    f.file(FileIntent::Navigate(NavigationIntent::Open));
    let HostEffect::ChooseSource(current) = f.effect() else {
        panic!()
    };
    assert!(
        f.provider
            .supply(&mut f.model.owner, context, old, f.directory.join("late"))
            .is_err()
    );
    assert_eq!(
        f.model
            .owner
            .app
            .projection(f.model.owner.session)
            .unwrap()
            .continuation,
        Some(current)
    );
    f.reply(current, EffectDecision::Cancel);
    f.effect();
}
#[test]
fn cancelled_active_read_keeps_current_document_and_retires_candidate() {
    let mut f = Fixture::new();
    f.stroke();
    let before = f.snapshot();
    let source = f.directory.join("source.nsketch");
    std::fs::write(&source, &before).unwrap();
    f.file(FileIntent::Navigate(NavigationIntent::Open));
    let HostEffect::ConfirmUnsaved { token, .. } = f.effect() else {
        panic!()
    };
    f.reply(token, EffectDecision::Unsaved(UnsavedChoice::Discard));
    f.choose(source.clone());
    let work = f.model.owner.app.take_work().unwrap().unwrap();
    let info = work.info();
    f.model.owner.app.cancel_operation(info.operation).unwrap();
    let mut result = Some(execute(Job {
        work,
        path: Some(source),
    }));
    f.model.owner.app.complete_work(&mut result).unwrap();
    assert!(
        f.model
            .owner
            .app
            .apply_ready(info.session, info.operation)
            .is_err()
    );
    f.drain();
    assert_eq!(f.snapshot(), before);
    assert!(f.model.owner.state().dirty);
}
