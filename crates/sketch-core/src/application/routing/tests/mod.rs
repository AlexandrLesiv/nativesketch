use super::*;
use crate::{Brush, Input, Sample};
mod effects;
mod navigation;
mod replacement;
mod saves;
const FIXTURE: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/portable-v1.nsketch"
));
fn brush(width: f64) -> BrushSettings {
    BrushSettings::try_from(Brush {
        rgba: [0.0, 0.0, 0.0, 1.0],
        width,
    })
    .unwrap()
}
fn app(edit: bool, export: bool) -> ApplicationSession {
    ApplicationSession::new(
        Capabilities {
            edit_document: edit,
            export_document: export,
        },
        if edit { ToolId::Pen } else { ToolId::Select },
        brush(4.0),
        Some(ValidatedDocument::decode(FIXTURE).unwrap()),
    )
    .unwrap()
}
fn queue() -> SaveCoordinator {
    SaveCoordinator::new(StorageLimits {
        jobs: 4,
        snapshot_bytes: 128 * 1024 * 1024,
    })
    .unwrap()
}
fn context(app: &ApplicationSession) -> CommandContext {
    let state = app.projection().editor;
    CommandContext {
        session: state.session,
        request: RequestId(42),
        interaction: match state.interaction {
            InteractionState::Idle => None,
            InteractionState::Drawing(id) | InteractionState::Transforming(id) => Some(id),
        },
    }
}
fn run(
    app: &mut ApplicationSession,
    queue: &mut SaveCoordinator,
    cmd: ApplicationCommand<'_>,
) -> ApplicationOutcome {
    app.dispatch(queue, context(app), cmd).unwrap()
}
fn edit(app: &mut ApplicationSession, queue: &mut SaveCoordinator, cmd: EditorCommand) {
    run(app, queue, ApplicationCommand::Edit(cmd));
}
fn file(
    app: &mut ApplicationSession,
    queue: &mut SaveCoordinator,
    intent: FileIntent,
) -> ApplicationEffect {
    run(app, queue, ApplicationCommand::File(intent))
        .effect
        .unwrap()
}
fn reply(
    app: &mut ApplicationSession,
    queue: &mut SaveCoordinator,
    token: EffectToken,
    decision: EffectDecision,
) -> ApplicationEffect {
    run(
        app,
        queue,
        ApplicationCommand::ResolveEffect(EffectReply { token, decision }),
    )
    .effect
    .unwrap()
}
fn draft(app: &mut ApplicationSession, queue: &mut SaveCoordinator) {
    edit(
        app,
        queue,
        EditorCommand::BeginStroke {
            pointer: PointerId(7),
            device: DeviceKind::Pen,
        },
    );
    let sample = Sample {
        id: 1,
        x: 10.0,
        y: 20.0,
        timestamp: 1.0,
        pressure: None,
    };
    run(
        app,
        queue,
        ApplicationCommand::Input(SampleBatch {
            pointer: PointerId(7),
            samples: &[
                Input::Actual(sample),
                Input::Predicted(Sample {
                    id: 2,
                    timestamp: 2.0,
                    ..sample
                }),
            ],
        }),
    );
}
fn stroke(app: &mut ApplicationSession, queue: &mut SaveCoordinator) {
    draft(app, queue);
    edit(app, queue, EditorCommand::CommitStroke(PointerId(7)));
}
fn bytes(app: &ApplicationSession) -> Vec<u8> {
    let QueryResult::Document(document) = app.query(EditorQuery::CommittedSnapshot).unwrap() else {
        panic!()
    };
    document.encode().unwrap()
}
fn prepared(
    app: &mut ApplicationSession,
    queue: &mut SaveCoordinator,
    destination: DestinationId,
) -> PreparedReplacement {
    let ApplicationEffect::ChooseSource(token) =
        file(app, queue, FileIntent::Navigate(NavigationIntent::Open))
    else {
        panic!()
    };
    let ApplicationEffect::PrepareReplacement(ticket) =
        reply(app, queue, token, EffectDecision::Source(destination))
    else {
        panic!()
    };
    ticket
        .prepare(Some(ValidatedDocument::decode(FIXTURE).unwrap()))
        .unwrap()
}
fn save_as(
    app: &mut ApplicationSession,
    queue: &mut SaveCoordinator,
    destination: DestinationId,
) -> SaveToken {
    let ApplicationEffect::ChooseDestination { token, .. } = file(app, queue, FileIntent::SaveAs)
    else {
        panic!()
    };
    let ApplicationEffect::SaveQueued(save) =
        reply(app, queue, token, EffectDecision::Destination(destination))
    else {
        panic!()
    };
    save
}
fn complete(queue: &mut SaveCoordinator, result: WriteOutcome) -> SaveCompletion {
    let job = queue.start_next().unwrap();
    let token = job.token();
    queue.finish(job.complete(result)).unwrap();
    queue.take_completion(token).unwrap()
}
fn written() -> WriteOutcome {
    WriteOutcome::Written(Durability::FileAndDirectorySynced)
}

mod lifecycle;

mod failure_ownership;
mod limits;
mod pending_binding;
