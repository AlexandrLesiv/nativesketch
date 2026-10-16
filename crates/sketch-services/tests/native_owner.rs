#[path = "native_runtime/support.rs"]
mod support;
use sketch_core::application::{routing::*, *};
use sketch_services::native_runtime::*;
use support::*;
#[test]
fn resolution_ready_preserves_new_stroke_and_saves_exact_resolved_admission() {
    let (mut runtime, window, root) = fixture();
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
    let path = root.join("drawing.nsketch");
    runtime
        .supply_location(window, context, token, 41, path.clone())
        .unwrap();
    begin(&mut runtime, window);
    let event = ready(&mut runtime);
    assert!(matches!(
        runtime.apply_event(window, event.id),
        Err(RuntimeError::Busy)
    ));
    assert!(matches!(
        runtime
            .session(window)
            .unwrap()
            .app
            .projection()
            .editor
            .interaction,
        InteractionState::Drawing(_)
    ));
    assert!(
        runtime
            .file_available(window, FileIntent::Navigate(NavigationIntent::Open))
            .is_err()
    );
    edit(
        &mut runtime,
        window,
        EditorCommand::CommitStroke(PointerId(7)),
    );
    let saved = runtime.apply_event(window, event.id).unwrap();
    assert!(matches!(saved.effect, IntentEffect::Save(_)));
    let event = ready(&mut runtime);
    runtime.apply_event(window, event.id).unwrap();
    let QueryResult::Document(snapshot) = runtime
        .session(window)
        .unwrap()
        .app
        .query(EditorQuery::CommittedSnapshot)
        .unwrap()
    else {
        panic!("snapshot")
    };
    assert_eq!(std::fs::read(path).unwrap(), snapshot.encode().unwrap());
    assert!(
        !runtime
            .session(window)
            .unwrap()
            .app
            .projection()
            .editor
            .document
            .dirty
    );
    close(runtime, window);
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn poisoned_detach_drains_accepted_write_and_releases_grant_once() {
    let (mut runtime, window, root) = fixture();
    begin(&mut runtime, window);
    edit(
        &mut runtime,
        window,
        EditorCommand::CommitStroke(PointerId(7)),
    );
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
    let path = root.join("poisoned.nsketch");
    runtime
        .supply_location(window, context, token, 41, path.clone())
        .unwrap();
    let event = ready(&mut runtime);
    runtime.apply_event(window, event.id).unwrap();
    runtime.detach(window, true).unwrap();
    runtime.request_shutdown().unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    let mut released = 0;
    while !runtime.free_ready().unwrap() {
        for event in runtime.poll(32).unwrap() {
            if matches!(event.kind, EventKind::GrantRelease) {
                assert_eq!(event.grant, 41);
                released += 1;
            }
            assert!(!matches!(event.kind, EventKind::OwnerReady));
            runtime.release_event(event.id).unwrap();
        }
        assert!(std::time::Instant::now() < deadline);
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    assert_eq!(released, 1);
    assert!(ValidatedDocument::decode(&std::fs::read(path).unwrap()).is_ok());
    std::fs::remove_dir_all(root).unwrap();
}
