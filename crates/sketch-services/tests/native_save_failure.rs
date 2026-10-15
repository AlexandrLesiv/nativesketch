#![cfg(feature = "native-test-controls")]
#[path = "native_runtime/support.rs"]
mod support;
use sketch_core::application::{routing::*, storage::SaveResult, *};
use sketch_services::native_runtime::*;
use support::*;
#[test]
fn failed_save_as_returns_correlated_save_as_retry_not_old_binding_save() {
    let (mut runtime, window, root) = fixture();
    let control = root.join("fault");
    std::fs::create_dir(&control).unwrap();
    runtime
        .arm_test(TestControl::new(control, 2, 2).unwrap())
        .unwrap();
    let event = ready(&mut runtime);
    runtime.release_event(event.id).unwrap();
    begin(&mut runtime, window);
    edit(
        &mut runtime,
        window,
        EditorCommand::CommitStroke(PointerId(7)),
    );
    let result = runtime
        .dispatch(
            window,
            ctx(&runtime, window),
            ApplicationCommand::File(FileIntent::SaveAs),
        )
        .unwrap();
    let IntentEffect::Destination(token, _) = result.effect else {
        panic!("chooser")
    };
    let path = root.join("failed.nsketch");
    runtime
        .supply_location(window, ctx(&runtime, window), token, 71, path.clone())
        .unwrap();
    let event = ready(&mut runtime);
    runtime.apply_event(window, event.id).unwrap();
    let event = ready(&mut runtime);
    let result = runtime.apply_event(window, event.id).unwrap();
    assert!(matches!(result.effect, IntentEffect::Failure(_)));
    assert!(matches!(
        result.outcome.save.unwrap().result,
        SaveResult::Failed(_)
    ));
    assert_eq!(result.retry_action, 21);
    assert_eq!(result.operation, event.operation);
    assert_eq!(result.request, 10);
    assert!(!path.exists());
    // The retry's Rust-selected intent issues a new chooser with a fresh snapshot admission.
    let result = runtime
        .dispatch(
            window,
            ctx(&runtime, window),
            ApplicationCommand::File(FileIntent::SaveAs),
        )
        .unwrap();
    let IntentEffect::Destination(token, _) = result.effect else {
        panic!("retry chooser")
    };
    runtime
        .supply_location(window, ctx(&runtime, window), token, 72, path)
        .unwrap();
    loop {
        let event = ready(&mut runtime);
        if matches!(event.kind, EventKind::OwnerReady) {
            runtime.apply_event(window, event.id).unwrap();
            break;
        }
        runtime.release_event(event.id).unwrap();
    }
    let event = ready(&mut runtime);
    runtime.apply_event(window, event.id).unwrap();
    close(runtime, window);
    std::fs::remove_dir_all(root).unwrap();
}
