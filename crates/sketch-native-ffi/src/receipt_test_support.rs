use sketch_core::{
    Input, Sample,
    application::{routing::*, *},
};
use sketch_services::native_runtime::*;
use std::time::{Duration, Instant};
pub(super) fn context(runtime: &NativeRuntime, window: u64) -> CommandContext {
    let state = runtime.session(window).unwrap().app.projection().editor;
    CommandContext {
        session: state.session,
        request: RequestId(42),
        interaction: match state.interaction {
            InteractionState::Idle => None,
            InteractionState::Drawing(id) | InteractionState::Transforming(id) => Some(id),
        },
    }
}
fn edit(runtime: &mut NativeRuntime, window: u64, command: EditorCommand) {
    runtime
        .dispatch(
            window,
            context(runtime, window),
            ApplicationCommand::Edit(command),
        )
        .unwrap();
}
pub(super) fn stroke(runtime: &mut NativeRuntime, window: u64) {
    edit(
        runtime,
        window,
        EditorCommand::BeginStroke {
            pointer: PointerId(1),
            device: DeviceKind::Pen,
        },
    );
    let samples = [Input::Actual(Sample {
        id: 1,
        x: 10.0,
        y: 20.0,
        timestamp: 1.0,
        pressure: None,
    })];
    runtime
        .dispatch(
            window,
            context(runtime, window),
            ApplicationCommand::Input(SampleBatch {
                pointer: PointerId(1),
                samples: &samples,
            }),
        )
        .unwrap();
    edit(runtime, window, EditorCommand::CommitStroke(PointerId(1)));
}
pub(super) fn ready(runtime: &mut NativeRuntime) -> NativeEvent {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if let Some(event) = runtime.poll(1).unwrap().first() {
            return *event;
        }
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(1));
    }
}
