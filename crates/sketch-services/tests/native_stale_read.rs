#[path = "native_runtime/support.rs"]
mod support;
use sketch_core::application::{routing::*, *};
use sketch_services::native_runtime::*;
use support::*;
#[test]
fn global_resolution_barrier_and_repeated_stale_reads_preserve_new_ink_and_release_leases() {
    let (mut runtime, window, root) = fixture();
    let other = runtime
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
    let QueryResult::Document(snapshot) = runtime
        .session(window)
        .unwrap()
        .app
        .query(EditorQuery::CommittedSnapshot)
        .unwrap()
    else {
        panic!("snapshot")
    };
    let path = root.join("read.nsketch");
    std::fs::write(&path, snapshot.encode().unwrap()).unwrap();
    for iteration in 0..3 {
        let result = runtime
            .dispatch(
                window,
                ctx(&runtime, window),
                ApplicationCommand::File(FileIntent::Navigate(NavigationIntent::Open)),
            )
            .unwrap();
        let IntentEffect::Source(token) = result.effect else {
            panic!("source")
        };
        runtime
            .supply_location(
                window,
                ctx(&runtime, window),
                token,
                100 + iteration,
                path.clone(),
            )
            .unwrap();
        assert_eq!(runtime.preparing_operations(window).unwrap(), 1);
        begin(&mut runtime, other);
        assert!(matches!(
            runtime.dispatch(
                other,
                ctx(&runtime, other),
                ApplicationCommand::File(FileIntent::SaveAs)
            ),
            Err(RuntimeError::Busy)
        ));
        assert!(matches!(
            runtime
                .session(other)
                .unwrap()
                .app
                .projection()
                .editor
                .interaction,
            InteractionState::Drawing(_)
        ));
        edit(&mut runtime, other, EditorCommand::CancelInteraction);
        let event = ready(&mut runtime);
        runtime.apply_event(window, event.id).unwrap();
        // Read is now based on the old revision. A real later commit must make installation stale.
        begin(&mut runtime, window);
        edit(
            &mut runtime,
            window,
            EditorCommand::CommitStroke(PointerId(7)),
        );
        let event = ready(&mut runtime);
        assert!(matches!(
            runtime.apply_event(window, event.id),
            Err(RuntimeError::Application(ApplicationFailure::StaleDocument))
        ));
        runtime.release_event(event.id).unwrap();
        assert_eq!(runtime.preparing_operations(window).unwrap(), 0);
        let release = ready(&mut runtime);
        assert!(matches!(release.kind, EventKind::GrantRelease));
        assert_eq!(release.grant, 100 + iteration);
        runtime.release_event(release.id).unwrap();
        edit(&mut runtime, window, EditorCommand::Undo);
    }
    runtime.detach(other, false).unwrap();
    close(runtime, window);
    std::fs::remove_dir_all(root).unwrap();
}
