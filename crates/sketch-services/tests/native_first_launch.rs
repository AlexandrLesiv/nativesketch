#[path = "native_runtime/support.rs"]
mod support;
use sketch_core::application::{routing::*, storage::SaveResult, *};
use sketch_services::{native_runtime::*, recovery::RecoveryId};
use support::*;
#[test]
fn first_launch_nested_private_roots_support_registration_and_real_checkpoint() {
    let (initial, w, fixture) = fixture();
    close(initial, w);
    let root = fixture.join("new-application/runtime");
    let mut runtime = NativeRuntime::new(RuntimeConfig {
        recovery_root: Some(root.join("Recovery")),
        diagnostics_root: Some(root.join("Diagnostics")),
        ..Default::default()
    })
    .unwrap();
    let w = runtime
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
    let session = runtime
        .session(w)
        .unwrap()
        .app
        .projection()
        .editor
        .session
        .get();
    runtime
        .register_recovery(
            w,
            session,
            RecoveryId::from_bytes([41; 16]).unwrap(),
            "Fresh".into(),
        )
        .unwrap();
    let e = ready(&mut runtime);
    assert!(e.failure.is_none());
    runtime.release_event(e.id).unwrap();
    assert_eq!(
        runtime.session(w).unwrap().recovery.as_ref().unwrap().state,
        2
    );
    begin(&mut runtime, w);
    edit(&mut runtime, w, EditorCommand::CommitStroke(PointerId(7)));
    std::thread::sleep(std::time::Duration::from_millis(2050));
    runtime.idle().unwrap();
    let e = ready(&mut runtime);
    let result = runtime.apply_event(w, e.id).unwrap();
    assert!(matches!(
        result.outcome.save.unwrap().result,
        SaveResult::Written(_)
    ));
    assert!(root.join("Recovery/sessions").is_dir());
    assert!(root.join("Diagnostics/diagnostics.0.jsonl").is_file());
    let result = runtime
        .dispatch(
            w,
            ctx(&runtime, w),
            ApplicationCommand::File(FileIntent::SaveAs),
        )
        .unwrap();
    let IntentEffect::Destination(token, _) = result.effect else {
        panic!("save chooser")
    };
    runtime
        .supply_location(
            w,
            ctx(&runtime, w),
            token,
            81,
            fixture.join("saved.nsketch"),
        )
        .unwrap();
    let e = ready(&mut runtime);
    runtime.apply_event(w, e.id).unwrap();
    let e = ready(&mut runtime);
    runtime.apply_event(w, e.id).unwrap();
    let e = ready(&mut runtime);
    runtime.release_event(e.id).unwrap();
    close(runtime, w);
    std::fs::remove_dir_all(fixture).unwrap();
}
