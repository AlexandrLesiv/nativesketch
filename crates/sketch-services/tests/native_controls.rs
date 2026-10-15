#![cfg(feature = "native-test-controls")]
#[path = "native_runtime/support.rs"]
mod support;
use sketch_core::application::{routing::*, storage::SaveResult, *};
use sketch_services::native_runtime::*;
use support::*;
#[test]
fn explicit_save_gate_keeps_immutable_a_while_owner_commits_b() {
    let (mut runtime, window, root) = fixture();
    let control = root.join("control");
    std::fs::create_dir(&control).unwrap();
    runtime
        .arm_test(TestControl::new(control.clone(), 2, 1).unwrap())
        .unwrap();
    let event = ready(&mut runtime);
    runtime.release_event(event.id).unwrap();
    begin(&mut runtime, window);
    edit(
        &mut runtime,
        window,
        EditorCommand::CommitStroke(PointerId(7)),
    );
    let QueryResult::Document(snapshot) = runtime
        .session(window)
        .unwrap()
        .app
        .query(EditorQuery::CommittedSnapshot)
        .unwrap()
    else {
        panic!("snapshot")
    };
    let expected = snapshot.encode().unwrap();
    let context = ctx(&runtime, window);
    let result = runtime
        .dispatch(
            window,
            context,
            ApplicationCommand::File(FileIntent::SaveAs),
        )
        .unwrap();
    let token = match result.effect {
        IntentEffect::Destination(token, _) => token,
        _ => panic!("chooser"),
    };
    let path = root.join("held.nsketch");
    runtime
        .supply_location(window, context, token, 41, path.clone())
        .unwrap();
    let event = ready(&mut runtime);
    runtime.apply_event(window, event.id).unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while !control.join("reached").exists() {
        runtime.poll(32).unwrap();
        assert!(std::time::Instant::now() < deadline);
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    begin(&mut runtime, window);
    edit(
        &mut runtime,
        window,
        EditorCommand::CommitStroke(PointerId(7)),
    );
    assert!(!path.exists());
    std::fs::write(control.join("release"), []).unwrap();
    let event = ready(&mut runtime);
    let applied = runtime.apply_event(window, event.id).unwrap();
    assert!(matches!(
        applied.outcome.save.unwrap().result,
        SaveResult::Written(_)
    ));
    assert_eq!(std::fs::read(&path).unwrap(), expected);
    assert!(
        runtime
            .session(window)
            .unwrap()
            .app
            .projection()
            .editor
            .document
            .dirty
    );
    let context = ctx(&runtime, window);
    runtime
        .dispatch(window, context, ApplicationCommand::File(FileIntent::Save))
        .unwrap();
    let event = ready(&mut runtime);
    runtime.apply_event(window, event.id).unwrap();
    close(runtime, window);
    std::fs::remove_dir_all(root).unwrap();
}
