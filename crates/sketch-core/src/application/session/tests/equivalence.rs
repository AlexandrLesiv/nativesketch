use super::*;
fn equivalent(legacy: &Editor, typed: &EditorSession) {
    assert_eq!(legacy.summary(), typed.view_state().document);
    let QueryResult::Document(snapshot) = typed.query(EditorQuery::CommittedSnapshot).unwrap()
    else {
        panic!()
    };
    assert_eq!(legacy.document(), snapshot.document());
    let QueryResult::Scene(scene) = typed.query(EditorQuery::Scene).unwrap() else {
        panic!()
    };
    assert_eq!(
        format!("{:?}", legacy.render()),
        format!("{:?}", scene.commands)
    );
    let QueryResult::SelectedBounds(bounds) = typed.query(EditorQuery::SelectedBounds).unwrap()
    else {
        panic!()
    };
    assert_eq!(legacy.state().bounds, bounds);
    if legacy.summary().interaction == 0 {
        assert_eq!(legacy.export().unwrap().0, snapshot.encode().unwrap());
    }
}
#[test]
fn generated_editable_sessions_match_legacy_document_history_and_scene() {
    for seed in 0..32 {
        let mut legacy = Editor::new();
        let mut typed = session(true, false);
        for stroke in 0..3 {
            let settings = brush(1.0 + (seed + stroke) as f64);
            execute(&mut typed, EditorCommand::SetBrush(settings));
            legacy.begin(42, 1, settings.brush()).unwrap();
            let ctx = begin(&mut typed);
            for id in 1..=8 {
                let actual = Sample {
                    x: (seed as f64 - 16.0) * id as f64,
                    ..sample(id)
                };
                let inputs = [
                    Input::Actual(actual),
                    Input::Predicted(Sample {
                        x: 9000.0,
                        ..sample(id + 1)
                    }),
                ];
                legacy.append(42, &inputs).unwrap();
                typed
                    .append(
                        ctx,
                        SampleBatch {
                            pointer: PointerId(42),
                            samples: &inputs,
                        },
                    )
                    .unwrap();
                equivalent(&legacy, &typed);
            }
            if (seed + stroke) % 3 == 0 {
                legacy.cancel();
                execute(&mut typed, EditorCommand::CancelInteraction);
            } else {
                legacy.commit(42).unwrap();
                execute(&mut typed, EditorCommand::CommitStroke(PointerId(42)));
            }
            equivalent(&legacy, &typed);
        }
        execute(&mut typed, EditorCommand::SetTool(ToolId::Select));
        legacy.select_cycle(1).unwrap();
        execute(&mut typed, EditorCommand::SelectCycle(1));
        equivalent(&legacy, &typed);
        legacy.resize_begin().unwrap();
        execute(&mut typed, EditorCommand::BeginTransform);
        for scale in [1.0, 0.5, 3.0] {
            legacy.resize_update(scale, 2.0, -4.0).unwrap();
            execute(
                &mut typed,
                EditorCommand::UpdateTransform {
                    scale,
                    anchor_x: 2.0,
                    anchor_y: -4.0,
                },
            );
            equivalent(&legacy, &typed);
        }
        legacy.resize_commit().unwrap();
        execute(&mut typed, EditorCommand::CommitTransform);
        equivalent(&legacy, &typed);
        legacy.delete_selected().unwrap();
        execute(&mut typed, EditorCommand::DeleteSelection);
        equivalent(&legacy, &typed);
        while legacy.summary().can_undo {
            legacy.undo().unwrap();
            execute(&mut typed, EditorCommand::Undo);
            equivalent(&legacy, &typed);
        }
        while legacy.summary().can_redo {
            legacy.redo().unwrap();
            execute(&mut typed, EditorCommand::Redo);
            equivalent(&legacy, &typed);
        }
        legacy.import(FIXTURE).unwrap();
        execute(
            &mut typed,
            EditorCommand::ReplaceDocument(ValidatedDocument::decode(FIXTURE).unwrap()),
        );
        equivalent(&legacy, &typed);
        legacy.clear().unwrap();
        execute(&mut typed, EditorCommand::NewDocument);
        equivalent(&legacy, &typed);
    }
}
