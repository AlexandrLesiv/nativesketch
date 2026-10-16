use crate::{
    provider_work::{Job, execute},
    test_support::Fixture,
};
use sketch_app::HostEffect;
use sketch_core::application::routing::*;
#[test]
fn unknown_path_alias_exports_cannot_start_out_of_order() {
    let mut f = Fixture::new();
    f.stroke();
    let path = f.directory.join("same.nsketch");
    f.file(FileIntent::ExportCopy);
    f.choose(path.clone());
    let older = f.model.owner.app.take_work().unwrap().unwrap();
    f.stroke();
    f.file(FileIntent::ExportCopy);
    f.choose(path.clone());
    assert!(f.model.owner.app.take_work().unwrap().is_none());
    let info = older.info();
    let mut result = Some(execute(Job {
        work: older,
        path: Some(path.clone()),
    }));
    f.model.owner.app.complete_work(&mut result).unwrap();
    let outcome = f
        .model
        .owner
        .app
        .apply_ready(info.session, info.operation)
        .unwrap();
    f.model.owner.accept(outcome);
    f.drain();
    assert_eq!(
        u64::from_le_bytes(std::fs::read(path).unwrap()[8..16].try_into().unwrap()),
        2
    );
    assert!(f.model.owner.state().dirty);
    assert_eq!(f.model.owner.app.stats().resource_leases, 0);
}
#[test]
fn close_waits_for_actual_write_then_retires_every_resource() {
    let mut f = Fixture::new();
    f.stroke();
    let path = f.directory.join("save.nsketch");
    f.file(FileIntent::SaveAs);
    f.choose(path.clone());
    let work = f.model.owner.app.take_work().unwrap().unwrap();
    let info = work.info();
    f.file(FileIntent::Navigate(NavigationIntent::Close));
    let HostEffect::AwaitWrites(token) = f.effect() else {
        panic!("close must await actual write")
    };
    assert!(!f.model.owner.app.stats().shutdown_ready());
    f.reply(token, EffectDecision::Cancel);
    f.effect();
    // Canceling close does not claim that the in-flight write stopped.
    assert_eq!(f.model.owner.app.stats().outstanding_saves, 1);
    let mut result = Some(execute(Job {
        work,
        path: Some(path),
    }));
    f.model.owner.app.complete_work(&mut result).unwrap();
    let outcome = f
        .model
        .owner
        .app
        .apply_ready(info.session, info.operation)
        .unwrap();
    f.model.owner.accept(outcome);
    f.drain();
    assert!(!f.model.owner.state().dirty);
    // Fixture shutdown performs real Close/ConfirmClose, detach, worker retirement and join.
}
