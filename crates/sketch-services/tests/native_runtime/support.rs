use sketch_core::application::{routing::*, *};
use sketch_core::{Brush, Input, Sample};
use sketch_services::native_runtime::*;
use std::{
    fs,
    path::PathBuf,
    time::{Duration, Instant},
};
pub fn fixture() -> (NativeRuntime, u64, PathBuf) {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
    let path = std::env::temp_dir().join(format!(
        "runtime-owner-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ));
    fs::create_dir(&path).unwrap();
    let path = path.canonicalize().unwrap();
    let mut runtime = NativeRuntime::new(RuntimeConfig::default()).unwrap();
    let window = runtime
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
    (runtime, window, path)
}
pub fn ctx(runtime: &NativeRuntime, window: u64) -> CommandContext {
    let state = runtime.session(window).unwrap().app.projection().editor;
    runtime
        .context(
            window,
            state.session.get(),
            10,
            match state.interaction {
                InteractionState::Idle => 0,
                InteractionState::Drawing(id) | InteractionState::Transforming(id) => id.get(),
            },
        )
        .unwrap()
}
pub fn edit(runtime: &mut NativeRuntime, window: u64, command: EditorCommand) {
    let context = ctx(runtime, window);
    runtime
        .dispatch(window, context, ApplicationCommand::Edit(command))
        .unwrap();
}
pub fn begin(runtime: &mut NativeRuntime, window: u64) {
    edit(
        runtime,
        window,
        EditorCommand::BeginStroke {
            pointer: PointerId(7),
            device: DeviceKind::Pen,
        },
    );
    let samples = [Input::Actual(Sample {
        id: 1,
        x: 5.0,
        y: 10.0,
        timestamp: 1.0,
        pressure: Some(0.5),
    })];
    let context = ctx(runtime, window);
    runtime
        .dispatch(
            window,
            context,
            ApplicationCommand::Input(SampleBatch {
                pointer: PointerId(7),
                samples: &samples,
            }),
        )
        .unwrap();
}
pub fn ready(runtime: &mut NativeRuntime) -> NativeEvent {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let events = runtime.poll(1).unwrap();
        if let Some(event) = events.first() {
            return *event;
        }
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(1));
    }
}
pub fn close(mut runtime: NativeRuntime, window: u64) {
    let context = ctx(&runtime, window);
    let result = runtime
        .dispatch(
            window,
            context,
            ApplicationCommand::File(FileIntent::Navigate(NavigationIntent::Close)),
        )
        .unwrap();
    let token = match result.effect {
        IntentEffect::CloseReady(token) => token,
        _ => panic!("expected clean close"),
    };
    let context = ctx(&runtime, window);
    runtime
        .reply(window, context, token, EffectDecision::ConfirmClose)
        .unwrap();
    runtime.detach(window, false).unwrap();
    runtime.request_shutdown().unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    while !runtime.free_ready().unwrap() {
        for event in runtime.poll(32).unwrap() {
            runtime.release_event(event.id).unwrap();
        }
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(1));
    }
}
