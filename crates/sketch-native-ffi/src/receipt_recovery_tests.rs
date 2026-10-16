use crate::receipt_test_support::{context, ready, stroke};
use crate::{
    boundary::{Runtime, Session},
    dto,
    runtime_api::*,
    session_api::*,
};
use sketch_core::{
    Brush,
    application::{routing::*, *},
};
use sketch_services::native_runtime::*;
use std::{
    fs,
    time::{Duration, Instant},
};
// FFI-001, SAVE-003: actual I/O returns an invalid report, then a real queued write
// must still complete during poisoned teardown. No in-flight job is fabricated away.
#[test]
fn returned_invalid_receipt_poison_still_drains_real_successor_and_all_leases() {
    let root = std::env::temp_dir().join(format!(
        "receipt-recovery-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir(&root).unwrap();
    let root = root.canonicalize().unwrap();
    let path = root.join("drawing.nsketch");
    let control = root.join("control");
    fs::create_dir(&control).unwrap();
    let mut core = NativeRuntime::new(RuntimeConfig::default()).unwrap();
    let window = core
        .create(
            Capabilities {
                edit_document: true,
                export_document: true,
            },
            ToolId::Pen,
            BrushSettings::try_from(Brush {
                rgba: [0.0, 0.0, 0.0, 1.0],
                width: 4.0,
            })
            .unwrap(),
        )
        .unwrap();
    let outcome = core
        .dispatch(
            window,
            context(&core, window),
            ApplicationCommand::File(FileIntent::SaveAs),
        )
        .unwrap();
    let IntentEffect::Destination(token, _) = outcome.effect else {
        panic!()
    };
    core.supply_location(window, context(&core, window), token, 700, path.clone())
        .unwrap();
    let event = ready(&mut core);
    core.apply_event(window, event.id).unwrap();
    let event = ready(&mut core);
    core.apply_event(window, event.id).unwrap();
    core.arm_test(TestControl::new(control.clone(), 9, 1).unwrap())
        .unwrap();
    let event = ready(&mut core);
    core.release_event(event.id).unwrap();
    stroke(&mut core, window);
    core.dispatch(
        window,
        context(&core, window),
        ApplicationCommand::File(FileIntent::Save),
    )
    .unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    while !control.join("reached").exists() {
        core.poll(32).unwrap();
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(1));
    }
    let first_bytes = fs::read(&path).unwrap(); // first physical write has completed
    stroke(&mut core, window);
    core.dispatch(
        window,
        context(&core, window),
        ApplicationCommand::File(FileIntent::Save),
    )
    .unwrap();
    let QueryResult::Document(snapshot) = core
        .session(window)
        .unwrap()
        .app
        .query(EditorQuery::CommittedSnapshot)
        .unwrap()
    else {
        panic!()
    };
    let expected = snapshot.encode().unwrap();
    assert_ne!(expected, first_bytes);
    assert!(core.state().unwrap().pending >= 2);
    let mut runtime = Box::into_raw(Box::new(Runtime {
        core,
        owner: std::thread::current().id(),
        poisoned: false,
    }));
    let mut session = Box::into_raw(Box::new(Session {
        runtime,
        window,
        scene_active: Default::default(),
    }));
    fs::write(control.join("release"), []).unwrap();
    // All native pointers are uniquely owned and used serially on their creating thread.
    unsafe {
        loop {
            let mut events = [dto::Event::default(); 32];
            let mut count = 0;
            let mut state = dto::RuntimeState::default();
            let status =
                sketch_runtime_v1_poll(runtime, events.as_mut_ptr(), 32, &mut count, &mut state);
            if status == 6 {
                break;
            }
            assert_eq!(status, 0);
            assert_eq!(count, 0);
            assert!(Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(1));
        }
        assert!((*runtime).poisoned);
        assert_eq!(
            sketch_session_v1_projection(runtime, session, &mut dto::Projection::default()),
            10
        );
        assert!(
            (*runtime)
                .core
                .session(window)
                .unwrap()
                .app
                .projection()
                .editor
                .document
                .dirty
        );
        assert_eq!(sketch_session_v1_detach(runtime, &mut session), 0);
        assert_eq!(sketch_runtime_v1_request_shutdown(runtime), 0);
        let mut releases = 0;
        while !runtime.is_null() {
            let mut events = [dto::Event::default(); 32];
            let mut count = 0;
            let mut state = dto::RuntimeState::default();
            assert_eq!(
                sketch_runtime_v1_poll(runtime, events.as_mut_ptr(), 32, &mut count, &mut state),
                0
            );
            for event in &events[..count as usize] {
                if event.grant_id == 700 {
                    releases += 1;
                }
                assert_eq!(sketch_runtime_v1_release_event(runtime, event.event_id), 0);
            }
            let result = sketch_runtime_v1_free(&mut runtime);
            assert!(result == 0 || result == 9);
            assert!(Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(1));
        }
        assert_eq!(releases, 1);
    }
    assert_eq!(fs::read(path).unwrap(), expected);
    fs::remove_dir_all(root).unwrap();
}
