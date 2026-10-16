#[path = "native_runtime/support.rs"]
mod support;
use sketch_core::application::{routing::*, *};
use sketch_services::native_runtime::*;
use support::*;
#[test]
fn failed_read_and_recovery_export_redacted_correlated_records() {
    let (initial, window, root) = fixture();
    close(initial, window);
    let mut runtime = NativeRuntime::new(RuntimeConfig {
        diagnostics_root: Some(root.join("logs")),
        build: 567,
        ..Default::default()
    })
    .unwrap();
    let window = runtime
        .create(
            Capabilities {
                edit_document: true,
                export_document: true,
            },
            ToolId::Pen,
            BrushSettings::try_from(sketch_core::Brush {
                rgba: [0.0, 0.0, 0.0, 1.0],
                width: 4.0,
            })
            .unwrap(),
        )
        .unwrap();
    let path = root.join("secret-document-name.nsketch");
    std::fs::write(&path, b"private malformed document").unwrap();
    let outcome = runtime
        .dispatch(
            window,
            ctx(&runtime, window),
            ApplicationCommand::File(FileIntent::Navigate(NavigationIntent::Open)),
        )
        .unwrap();
    let IntentEffect::Source(token) = outcome.effect else {
        panic!("chooser")
    };
    runtime
        .supply_location(window, ctx(&runtime, window), token, 31, path)
        .unwrap();
    let event = ready(&mut runtime);
    runtime.apply_event(window, event.id).unwrap();
    let event = ready(&mut runtime);
    let outcome = runtime.apply_event(window, event.id).unwrap();
    assert!(matches!(outcome.effect, IntentEffect::Failure(_)));
    assert_eq!(outcome.request, 10);
    assert_eq!(outcome.operation_kind, 204);
    assert_eq!(outcome.operation, event.operation);
    runtime.list_recoveries(444).unwrap();
    loop {
        let event = ready(&mut runtime);
        let target = event.request == 444;
        if target {
            assert!(event.failure.is_some());
        }
        runtime.release_event(event.id).unwrap();
        if target {
            break;
        }
    }
    let exported = root.join("export.jsonl");
    runtime
        .export_diagnostics(555, 32, exported.clone())
        .unwrap();
    loop {
        let event = ready(&mut runtime);
        let complete = event.request == 555;
        assert!(event.failure.is_none());
        runtime.release_event(event.id).unwrap();
        if complete {
            break;
        }
    }
    let contents = std::fs::read_to_string(exported).unwrap();
    assert!(!contents.contains("secret-document"));
    assert!(!contents.contains("private malformed"));
    assert!(contents.lines().any(|line| line.contains("\"request\":10,")
        && line.contains("\"code\":11,")
        && line.contains("\"failure\":1,")));
    assert!(
        contents
            .lines()
            .any(|line| line.contains("\"request\":444,") && line.contains("\"code\":11,"))
    );
    assert!(contents.contains(&format!("\"operation_id\":{},", outcome.operation)));
    begin(&mut runtime, window);
    edit(&mut runtime, window, EditorCommand::CancelInteraction);
    close(runtime, window);
    std::fs::remove_dir_all(root).unwrap();
}
