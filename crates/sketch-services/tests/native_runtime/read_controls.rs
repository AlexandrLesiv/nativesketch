use super::support::*;
use sketch_core::application::{routing::*, storage::StorageFailure, *};
use sketch_services::native_runtime::*;
use std::{
    fs,
    path::Path,
    time::{Duration, Instant},
};

fn snapshot(runtime: &NativeRuntime, window: u64) -> Vec<u8> {
    let QueryResult::Document(document) = runtime
        .session(window)
        .unwrap()
        .app
        .query(EditorQuery::CommittedSnapshot)
        .unwrap()
    else {
        panic!("snapshot")
    };
    document.encode().unwrap()
}

fn open(runtime: &mut NativeRuntime, window: u64, path: &Path) {
    let result = runtime
        .dispatch(
            window,
            ctx(runtime, window),
            ApplicationCommand::File(FileIntent::Navigate(NavigationIntent::Open)),
        )
        .unwrap();
    let IntentEffect::Source(token) = result.effect else {
        panic!("source")
    };
    runtime
        .supply_location(window, ctx(runtime, window), token, 81, path.to_owned())
        .unwrap();
    let event = ready(runtime);
    runtime.apply_event(window, event.id).unwrap();
}

fn held_read(stale: bool) {
    let (mut runtime, window, root) = fixture();
    begin(&mut runtime, window);
    edit(
        &mut runtime,
        window,
        EditorCommand::CommitStroke(PointerId(7)),
    );
    let expected = snapshot(&runtime, window);
    edit(&mut runtime, window, EditorCommand::Undo);
    let original = snapshot(&runtime, window);
    assert_ne!(expected, original);
    let path = root.join("source.nsketch");
    fs::write(&path, &expected).unwrap();
    let control = root.join("control");
    fs::create_dir(&control).unwrap();
    runtime
        .arm_test(TestControl::new(control.clone(), 8, 1).unwrap())
        .unwrap();
    let event = ready(&mut runtime);
    runtime.release_event(event.id).unwrap();
    open(&mut runtime, window, &path);
    let deadline = Instant::now() + Duration::from_secs(5);
    while !control.join("reached").exists() {
        assert!(runtime.poll(32).unwrap().is_empty());
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(1));
    }
    assert!(runtime.poll(32).unwrap().is_empty());
    assert_eq!(runtime.preparing_operations(window).unwrap(), 1);
    assert_eq!(snapshot(&runtime, window), original);
    // The held value is already validated: modifying the file cannot replace that candidate.
    fs::write(&path, b"corrupt after validation").unwrap();
    if stale {
        begin(&mut runtime, window);
        edit(
            &mut runtime,
            window,
            EditorCommand::CommitStroke(PointerId(7)),
        );
    }
    let before_release = snapshot(&runtime, window);
    fs::write(control.join("release"), []).unwrap();
    let event = ready(&mut runtime);
    let result = runtime.apply_event(window, event.id);
    if stale {
        assert!(matches!(
            result,
            Err(RuntimeError::Application(ApplicationFailure::StaleDocument))
        ));
        assert_eq!(snapshot(&runtime, window), before_release);
        runtime.release_event(event.id).unwrap();
        let release = ready(&mut runtime);
        assert!(matches!(release.kind, EventKind::GrantRelease));
        runtime.release_event(release.id).unwrap();
        edit(&mut runtime, window, EditorCommand::Undo);
    } else {
        result.unwrap();
        assert_eq!(snapshot(&runtime, window), expected);
    }
    assert_eq!(runtime.preparing_operations(window).unwrap(), 0);
    close(runtime, window);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn validated_read_gate_holds_owned_candidate_until_release() {
    held_read(false);
}

#[test]
fn validated_read_gate_rejects_candidate_after_owner_edit() {
    held_read(true);
}

#[test]
fn read_validation_failure_is_one_shot_and_preserves_live_document() {
    let (mut runtime, window, root) = fixture();
    let original = snapshot(&runtime, window);
    let path = root.join("source.nsketch");
    fs::write(&path, &original).unwrap();
    let control = root.join("control");
    fs::create_dir(&control).unwrap();
    runtime
        .arm_test(TestControl::new(control.clone(), 8, 2).unwrap())
        .unwrap();
    let event = ready(&mut runtime);
    runtime.release_event(event.id).unwrap();
    // Invalid reads must not consume the successful-validation boundary.
    fs::write(&path, b"invalid document").unwrap();
    open(&mut runtime, window, &path);
    let event = ready(&mut runtime);
    let result = runtime.apply_event(window, event.id).unwrap();
    assert!(matches!(
        result.effect,
        IntentEffect::Failure(ApplicationFailure::Write(StorageFailure::Encode))
    ));
    assert!(!control.join("reached").exists());
    assert_eq!(snapshot(&runtime, window), original);
    let release = ready(&mut runtime);
    runtime.release_event(release.id).unwrap();
    fs::write(&path, &original).unwrap();
    open(&mut runtime, window, &path);
    let event = ready(&mut runtime);
    assert!(control.join("reached").exists());
    let result = runtime.apply_event(window, event.id).unwrap();
    assert!(matches!(
        result.effect,
        IntentEffect::Failure(ApplicationFailure::Write(StorageFailure::Access))
    ));
    assert_eq!(snapshot(&runtime, window), original);
    let release = ready(&mut runtime);
    runtime.release_event(release.id).unwrap();
    open(&mut runtime, window, &path);
    let event = ready(&mut runtime);
    runtime.apply_event(window, event.id).unwrap();
    assert_eq!(snapshot(&runtime, window), original);
    close(runtime, window);
    fs::remove_dir_all(root).unwrap();
}
