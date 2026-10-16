use super::*;
use crate::{Bounds, Document, Stroke, Transform};
fn bounds(editor: &EditorSession) -> Option<Bounds> {
    let QueryResult::DocumentBounds(value) = editor.query(EditorQuery::DocumentBounds).unwrap()
    else {
        panic!()
    };
    value
}
fn rendered_bounds(editor: &EditorSession) -> Option<Bounds> {
    let QueryResult::Scene(scene) = editor.query(EditorQuery::Scene).unwrap() else {
        panic!()
    };
    let mut low = [f64::INFINITY; 2];
    let mut high = [f64::NEG_INFINITY; 2];
    for path in scene.commands {
        let count = match path.verb {
            0 | 1 => 1,
            2 => 3,
            3 => 0,
            _ => panic!(),
        };
        for pair in path.points[..count * 2].chunks_exact(2) {
            for axis in 0..2 {
                low[axis] = low[axis].min(pair[axis]);
                high[axis] = high[axis].max(pair[axis]);
            }
        }
    }
    low[0].is_finite().then_some(Bounds {
        x: low[0],
        y: low[1],
        width: high[0] - low[0],
        height: high[1] - low[1],
    })
}
fn assert_render_equivalent(editor: &EditorSession) {
    let actual = bounds(editor);
    let expected = rendered_bounds(editor);
    match (actual, expected) {
        (Some(a), Some(b)) => {
            for (x, y) in [a.x, a.y, a.width, a.height]
                .into_iter()
                .zip([b.x, b.y, b.width, b.height])
            {
                assert!(
                    (x - y).abs() <= f64::EPSILON * 8.0 * x.abs().max(y.abs()).max(1.0),
                    "{x} vs {y}"
                );
            }
        }
        (None, None) => {}
        _ => panic!("empty mismatch"),
    }
}
// RENDER-001 / VIEW-001: full f64 outlines, independent from native Fit zoom policy.
#[test]
fn full_committed_bounds_match_outlines_for_pressure_transforms_and_far_coordinates() {
    let document = Document {
        strokes: [
            Transform {
                scale: 123.25,
                tx: 900_000_000_000.125,
                ty: 80.25,
            },
            Transform {
                scale: 0.001,
                tx: -700_000_000_000.375,
                ty: -60.5,
            },
        ]
        .into_iter()
        .enumerate()
        .map(|(index, transform)| Stroke {
            id: index as u64 + 1,
            device: 2,
            pointer_id: 7,
            brush: Brush {
                rgba: [0.0, 0.0, 0.0, 1.0],
                width: 4.25,
            },
            transform,
            samples: (1..=3)
                .map(|id| Sample {
                    x: id as f64 * 8.125,
                    y: id as f64 * -7.25,
                    pressure: match id {
                        1 => Some(0.0),
                        2 => Some(1.0),
                        _ => None,
                    },
                    ..sample(id)
                })
                .collect(),
        })
        .collect(),
    };
    let bytes = crate::persistence::encode(&document);
    for editable in [false, true] {
        let editor = EditorSession::new(
            Capabilities {
                edit_document: editable,
                export_document: false,
            },
            ToolId::Select,
            brush(4.0),
            Some(ValidatedDocument::decode(&bytes).unwrap()),
        )
        .unwrap();
        assert_render_equivalent(&editor);
        let b = bounds(&editor).unwrap();
        assert!(b.width > 1e12);
        assert_ne!(b.x, (b.x as f32) as f64);
    }
}
#[test]
fn bounds_exclude_live_actual_predictions_and_transform_preview_without_mutation() {
    let mut editor = session(true, true);
    let original = bounds(&editor);
    let ctx = begin(&mut editor);
    editor
        .append(
            ctx,
            SampleBatch {
                pointer: PointerId(42),
                samples: &[
                    Input::Actual(Sample {
                        x: 1e8,
                        ..sample(1)
                    }),
                    Input::Predicted(Sample {
                        x: -1e8,
                        ..sample(2)
                    }),
                ],
            },
        )
        .unwrap();
    let before = editor.view_state();
    assert_eq!(bounds(&editor), original);
    assert_eq!(editor.view_state(), before);
    execute(&mut editor, EditorCommand::CancelInteraction);
    execute(&mut editor, EditorCommand::SetTool(ToolId::Select));
    execute(&mut editor, EditorCommand::SelectCycle(1));
    execute(&mut editor, EditorCommand::BeginTransform);
    execute(
        &mut editor,
        EditorCommand::UpdateTransform {
            scale: 2.0,
            anchor_x: 0.0,
            anchor_y: 0.0,
        },
    );
    assert_eq!(bounds(&editor), original);
    execute(&mut editor, EditorCommand::CancelInteraction);
    assert_eq!(bounds(&editor), original);
}
#[test]
fn committed_bounds_follow_history_deletion_new_and_replacement() {
    let mut editor = session(true, false);
    assert_eq!(bounds(&editor), None);
    execute(
        &mut editor,
        EditorCommand::ReplaceDocument(ValidatedDocument::decode(FIXTURE).unwrap()),
    );
    let original = bounds(&editor);
    assert_render_equivalent(&editor);
    execute(&mut editor, EditorCommand::SetTool(ToolId::Select));
    execute(&mut editor, EditorCommand::SelectCycle(1));
    execute(&mut editor, EditorCommand::BeginTransform);
    execute(
        &mut editor,
        EditorCommand::UpdateTransform {
            scale: 2.0,
            anchor_x: 0.0,
            anchor_y: 0.0,
        },
    );
    execute(&mut editor, EditorCommand::CommitTransform);
    let scaled = bounds(&editor);
    assert_ne!(scaled, original);
    assert_render_equivalent(&editor);
    execute(&mut editor, EditorCommand::Undo);
    assert_eq!(bounds(&editor), original);
    execute(&mut editor, EditorCommand::Redo);
    assert_eq!(bounds(&editor), scaled);
    execute(&mut editor, EditorCommand::DeleteSelection);
    assert_render_equivalent(&editor);
    execute(&mut editor, EditorCommand::Undo);
    assert_eq!(bounds(&editor), scaled);
    execute(&mut editor, EditorCommand::NewDocument);
    assert_eq!(bounds(&editor), None);
}
