pub use sketch_core::application::*;
pub use sketch_core::*;
pub use std::sync::Arc;

pub fn editor() -> EditorSession {
    EditorSession::new(
        Capabilities {
            edit_document: true,
            export_document: true,
        },
        ToolId::Pen,
        BrushSettings::try_from(Brush {
            rgba: [0.2, 0.4, 0.6, 0.5],
            width: 4.0,
        })
        .unwrap(),
        None,
    )
    .unwrap()
}
pub fn context(editor: &EditorSession) -> CommandContext {
    let state = editor.view_state();
    CommandContext {
        session: state.session,
        request: RequestId(1),
        interaction: match state.interaction {
            InteractionState::Idle => None,
            InteractionState::Drawing(id) | InteractionState::Transforming(id) => Some(id),
        },
    }
}
pub fn execute(editor: &mut EditorSession, command: EditorCommand) {
    editor.execute(context(editor), command).unwrap();
}
pub fn sample(id: u64, x: f64, y: f64) -> Sample {
    Sample {
        id,
        x,
        y,
        timestamp: id as f64,
        pressure: Some(0.7),
    }
}
pub fn append(editor: &mut EditorSession, inputs: &[Input]) {
    editor
        .append(
            context(editor),
            SampleBatch {
                pointer: PointerId(1),
                samples: inputs,
            },
        )
        .unwrap();
}
pub fn begin(editor: &mut EditorSession) {
    execute(
        editor,
        EditorCommand::BeginStroke {
            pointer: PointerId(1),
            device: DeviceKind::Pen,
        },
    );
}
pub fn stroke(editor: &mut EditorSession, x: f64) {
    begin(editor);
    append(
        editor,
        &[
            Input::Actual(sample(1, x, 20.0)),
            Input::Actual(sample(2, x + 10.0, 20.0)),
        ],
    );
    execute(editor, EditorCommand::CommitStroke(PointerId(1)));
}
pub fn viewport() -> RenderViewport {
    RenderViewport {
        x: -100.0,
        y: -100.0,
        width: 2000.0,
        height: 2000.0,
        margin: 1.0,
    }
}
pub fn bytes(editor: &EditorSession) -> Vec<u8> {
    let QueryResult::Document(snapshot) = editor.query(EditorQuery::CommittedSnapshot).unwrap()
    else {
        panic!()
    };
    snapshot.encode().unwrap()
}
pub fn commands(scene: &RenderScene) -> Vec<PathCommand> {
    let preview = scene.transform_preview.as_ref();
    let mut strokes: Vec<_> = scene
        .committed
        .iter()
        .filter(|stroke| !preview.is_some_and(|p| p.stroke_id == stroke.stroke_id))
        .collect();
    if let Some(stroke) = preview.and_then(|p| p.replacement.as_ref()) {
        strokes.push(stroke);
    }
    if let Some(stroke) = scene.live.as_ref() {
        strokes.push(stroke);
    }
    strokes.sort_by_key(|stroke| stroke.order);
    strokes
        .into_iter()
        .flat_map(|stroke| {
            stroke.geometry.commands.iter().map(move |command| {
                let mut command = *command;
                let pairs = match command.verb {
                    0 | 1 => 1,
                    2 => 3,
                    3 => 0,
                    _ => panic!(),
                };
                for point in command.points[..pairs * 2].chunks_exact_mut(2) {
                    point[0] = point[0] * stroke.transform.scale + stroke.transform.tx;
                    point[1] = point[1] * stroke.transform.scale + stroke.transform.ty;
                }
                command
            })
        })
        .collect()
}
pub fn equivalent(left: &[PathCommand], right: &[PathCommand]) {
    assert_eq!(left.len(), right.len());
    for (a, b) in left.iter().zip(right) {
        assert_eq!((a.stroke_id, a.verb, a.rgba), (b.stroke_id, b.verb, b.rgba));
        for (x, y) in a.points.iter().zip(b.points) {
            assert!((x - y).abs() <= 1e-9_f64.max(y.abs() * 1e-14), "{x} != {y}");
        }
    }
}
pub fn reference(editor: &EditorSession, scene: &RenderScene) {
    let QueryResult::Scene(old) = editor.query(EditorQuery::Scene).unwrap() else {
        panic!()
    };
    equivalent(&commands(scene), &old.commands);
}
