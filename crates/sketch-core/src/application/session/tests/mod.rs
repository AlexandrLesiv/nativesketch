use super::*;
use crate::{Brush, Input, Sample};
mod capabilities;
mod capacity;
mod equivalence;
mod lifecycle;
mod transactions;
const FIXTURE: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/portable-v1.nsketch"
));
fn brush(width: f64) -> BrushSettings {
    BrushSettings::try_from(Brush {
        rgba: [0.2, 0.3, 0.4, 0.5],
        width,
    })
    .unwrap()
}
fn session(edit: bool, initial: bool) -> EditorSession {
    EditorSession::new(
        Capabilities {
            edit_document: edit,
            export_document: false,
        },
        if edit { ToolId::Pen } else { ToolId::Select },
        brush(6.0),
        initial.then(|| ValidatedDocument::decode(FIXTURE).unwrap()),
    )
    .unwrap()
}
fn context(session: &EditorSession) -> CommandContext {
    let state = session.view_state();
    CommandContext {
        session: state.session,
        request: RequestId(17),
        interaction: match state.interaction {
            InteractionState::Idle => None,
            InteractionState::Drawing(id) | InteractionState::Transforming(id) => Some(id),
        },
    }
}
fn execute(session: &mut EditorSession, command: EditorCommand) -> CommandOutcome {
    session.execute(context(session), command).unwrap()
}
fn sample(id: u64) -> Sample {
    Sample {
        id,
        x: id as f64 * 10.0,
        y: 20.0,
        timestamp: id as f64,
        pressure: (id % 2 == 1).then_some(0.5),
    }
}
fn begin(session: &mut EditorSession) -> CommandContext {
    execute(
        session,
        EditorCommand::BeginStroke {
            pointer: PointerId(42),
            device: DeviceKind::Pen,
        },
    );
    context(session)
}
fn snapshot(session: &EditorSession) -> Vec<u8> {
    let QueryResult::Document(snapshot) = session.query(EditorQuery::CommittedSnapshot).unwrap()
    else {
        panic!()
    };
    snapshot.encode().unwrap()
}
fn scene(session: &EditorSession) -> Vec<[u64; 12]> {
    let QueryResult::Scene(scene) = session.query(EditorQuery::Scene).unwrap() else {
        panic!()
    };
    scene
        .commands
        .into_iter()
        .map(|c| {
            let mut bits = [0; 12];
            bits[0] = c.stroke_id;
            bits[1] = u64::from(c.verb);
            for (slot, value) in bits[2..].iter_mut().zip(c.rgba.into_iter().chain(c.points)) {
                *slot = value.to_bits();
            }
            bits
        })
        .collect()
}

mod projections;
