use super::*;
use crate::application::*;
use crate::{Brush, Input, Sample};
mod acknowledgement;
mod admission;
mod binding_order;
mod cancellation;
mod ordering;
const FIXTURE: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/portable-v1.nsketch"
));
const DESTINATION: DestinationId = DestinationId(1);
fn session() -> EditorSession {
    EditorSession::new(
        Capabilities {
            edit_document: true,
            export_document: true,
        },
        ToolId::Pen,
        BrushSettings::try_from(Brush {
            rgba: [0.0, 0.0, 0.0, 1.0],
            width: 3.0,
        })
        .unwrap(),
        Some(ValidatedDocument::decode(FIXTURE).unwrap()),
    )
    .unwrap()
}
fn coordinator() -> SaveCoordinator {
    SaveCoordinator::new(StorageLimits {
        jobs: 4,
        snapshot_bytes: 128 * 1024 * 1024,
    })
    .unwrap()
}
fn context(session: &EditorSession) -> CommandContext {
    let state = session.view_state();
    CommandContext {
        session: state.session,
        request: RequestId(1),
        interaction: match state.interaction {
            InteractionState::Idle => None,
            InteractionState::Drawing(id) | InteractionState::Transforming(id) => Some(id),
        },
    }
}
fn execute(session: &mut EditorSession, command: EditorCommand) {
    session.execute(context(session), command).unwrap();
}
fn draft(session: &mut EditorSession) {
    execute(
        session,
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
    session
        .append(
            context(session),
            SampleBatch {
                pointer: PointerId(7),
                samples: &[
                    Input::Actual(sample),
                    Input::Predicted(Sample {
                        id: 2,
                        timestamp: 2.0,
                        ..sample
                    }),
                ],
            },
        )
        .unwrap();
}
fn stroke(session: &mut EditorSession) {
    draft(session);
    execute(session, EditorCommand::CommitStroke(PointerId(7)));
}
fn request(
    coordinator: &mut SaveCoordinator,
    session: &mut EditorSession,
    purpose: SavePurpose,
) -> SaveToken {
    coordinator
        .request(session, RequestId(42), purpose, DESTINATION)
        .unwrap()
}
fn written(coordinator: &mut SaveCoordinator) -> SaveCompletion {
    let job = coordinator.start_next().unwrap();
    let token = job.token();
    coordinator
        .finish(job.complete(WriteOutcome::Written(Durability::FileAndDirectorySynced)))
        .unwrap();
    coordinator.take_completion(token).unwrap()
}
fn bound() -> EditorSession {
    let mut session = session();
    session.bind_loaded_destination(DESTINATION).unwrap();
    session
}
