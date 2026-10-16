use crate::{dto, runtime_api::*, session_api::*};
use sketch_core::application::{DeviceKind, EditorCommand, PointerId, routing::ApplicationCommand};
#[test]
fn caught_boundary_panic_preserves_dirty_detach_guard_then_allows_terminal_cleanup() {
    // All pointers originate from this module's constructors and remain uniquely owned here.
    unsafe {
        let config = dto::RuntimeConfig {
            struct_size: std::mem::size_of::<dto::RuntimeConfig>() as u32,
            max_sessions: 0,
            max_operations: 0,
            max_events: 0,
            snapshot_bytes: 0,
            build_id: 0,
            recovery_root: dto::Text {
                bytes: std::ptr::null(),
                length: 0,
            },
            diagnostics_root: dto::Text {
                bytes: std::ptr::null(),
                length: 0,
            },
        };
        let mut runtime = std::ptr::null_mut();
        assert_eq!(sketch_runtime_v1_create(&config, &mut runtime), 0);
        let config = dto::SessionConfig {
            capabilities: 3,
            tool: 1,
            brush: dto::Brush {
                red: 0.0,
                green: 0.0,
                blue: 0.0,
                alpha: 1.0,
                width: 4.0,
            },
        };
        let mut session = std::ptr::null_mut();
        assert_eq!(sketch_session_v1_create(runtime, &config, &mut session), 0);
        let window = (*session).window;
        let projection = (*runtime).core.session(window).unwrap().app.projection();
        let context = (*runtime)
            .core
            .context(window, projection.editor.session.get(), 42, 0)
            .unwrap();
        (*runtime)
            .core
            .dispatch(
                window,
                context,
                ApplicationCommand::Edit(EditorCommand::BeginStroke {
                    pointer: PointerId(1),
                    device: DeviceKind::Pen,
                }),
            )
            .unwrap();
        let state = (*runtime)
            .core
            .session(window)
            .unwrap()
            .app
            .projection()
            .editor;
        let interaction = match state.interaction {
            sketch_core::application::InteractionState::Drawing(id) => id.get(),
            _ => panic!("drawing"),
        };
        let context = (*runtime)
            .core
            .context(window, state.session.get(), 43, interaction)
            .unwrap();
        let samples = [sketch_core::Input::Actual(sketch_core::Sample {
            id: 1,
            x: 1.0,
            y: 1.0,
            timestamp: 1.0,
            pressure: None,
        })];
        (*runtime)
            .core
            .dispatch(
                window,
                context,
                ApplicationCommand::Input(sketch_core::application::SampleBatch {
                    pointer: PointerId(1),
                    samples: &samples,
                }),
            )
            .unwrap();
        (*runtime)
            .core
            .dispatch(
                window,
                context,
                ApplicationCommand::Edit(EditorCommand::CommitStroke(PointerId(1))),
            )
            .unwrap();
        // A dirty unapproved document cannot be silently retired through the normal API.
        assert_ne!(sketch_session_v1_detach(runtime, &mut session), 0);
        assert!(!session.is_null());
        #[cfg(not(feature = "native-test-controls"))]
        let poison = crate::boundary::runtime(runtime, false, |_| {
            panic!("injected internal boundary failure")
        });
        #[cfg(feature = "native-test-controls")]
        let poison = crate::test_api::sketch_runtime_v1_test_poison(runtime);
        assert_eq!(poison, 6);
        assert_eq!(
            sketch_session_v1_projection(runtime, session, &mut dto::Projection::default()),
            10
        );
        assert_eq!(sketch_session_v1_detach(runtime, &mut session), 0);
        assert!(session.is_null());
        assert_eq!(sketch_runtime_v1_request_shutdown(runtime), 0);
        let mut stale = dto::RuntimeState {
            wake_after_ms: u64::MAX,
            ..Default::default()
        };
        assert_eq!(sketch_runtime_v1_idle(runtime, &mut stale), 10);
        assert_eq!(stale.needs_poll, 0);
        assert_eq!(stale.wake_after_ms, u64::MAX);
        let mut fresh = dto::RuntimeState::default();
        assert_eq!(sketch_runtime_v1_state(runtime, &mut fresh), 0);
        assert_eq!(fresh.needs_poll, 1);
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        loop {
            let mut events = [dto::Event::default(); 32];
            let mut count = 0;
            let mut state = dto::RuntimeState::default();
            assert_eq!(
                sketch_runtime_v1_poll(runtime, events.as_mut_ptr(), 32, &mut count, &mut state),
                0
            );
            for event in &events[..count as usize] {
                assert_eq!(sketch_runtime_v1_release_event(runtime, event.event_id), 0);
            }
            if sketch_runtime_v1_free(&mut runtime) == 0 {
                break;
            }
            assert!(std::time::Instant::now() < deadline);
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
        assert!(runtime.is_null());
    }
}
