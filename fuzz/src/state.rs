use sketch_core::application::*;

pub fn context(session: &EditorSession, request: u64) -> CommandContext {
    let state = session.view_state();
    CommandContext {
        session: state.session,
        request: RequestId(request),
        interaction: match state.interaction {
            InteractionState::Idle => None,
            InteractionState::Drawing(id) | InteractionState::Transforming(id) => Some(id),
        },
    }
}

pub fn snapshot(session: &EditorSession) -> Vec<u8> {
    let QueryResult::Document(document) = session.query(EditorQuery::CommittedSnapshot).unwrap()
    else {
        unreachable!()
    };
    document.encode().unwrap()
}

pub fn scene(session: &EditorSession) -> Vec<[u64; 12]> {
    let QueryResult::Scene(scene) = session.query(EditorQuery::Scene).unwrap() else {
        unreachable!()
    };
    scene
        .commands
        .into_iter()
        .map(|command| {
            let mut values = [0; 12];
            values[0] = command.stroke_id;
            values[1] = u64::from(command.verb);
            for (slot, value) in values[2..]
                .iter_mut()
                .zip(command.rgba.into_iter().chain(command.points))
            {
                assert!(value.is_finite());
                *slot = value.to_bits();
            }
            values
        })
        .collect()
}
