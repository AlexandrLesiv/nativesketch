use super::*;
#[test]
fn generation_exhaustion_rejects_before_brush_document_or_preview_mutation() {
    let mut editor = session(true, true);
    editor.generation = SceneGeneration(u64::MAX);
    let before = editor.view_state();
    let bytes = snapshot(&editor);
    for command in [
        EditorCommand::SetBrush(brush(50.0)),
        EditorCommand::NewDocument,
        EditorCommand::BeginStroke {
            pointer: PointerId(42),
            device: DeviceKind::Pen,
        },
    ] {
        assert_eq!(
            editor.execute(context(&editor), command).unwrap_err().kind,
            FailureKind::ResourceLimit
        );
        assert_eq!(editor.view_state(), before);
        assert_eq!(snapshot(&editor), bytes);
    }
    let mut active = session(true, false);
    let ctx = begin(&mut active);
    active.generation = SceneGeneration(u64::MAX);
    assert_eq!(
        active
            .append(
                ctx,
                SampleBatch {
                    pointer: PointerId(42),
                    samples: &[Input::Actual(sample(1))]
                }
            )
            .unwrap_err()
            .kind,
        FailureKind::ResourceLimit
    );
    assert!(scene(&active).is_empty());
    assert_eq!(
        active.view_state().scene_generation,
        SceneGeneration(u64::MAX)
    );
}
#[test]
fn exhausted_interaction_identity_never_starts_or_reuses_a_transaction() {
    let mut editor = session(true, false);
    editor.next_interaction = u64::MAX;
    let before = editor.view_state();
    assert_eq!(
        editor
            .execute(
                context(&editor),
                EditorCommand::BeginStroke {
                    pointer: PointerId(42),
                    device: DeviceKind::Pen
                }
            )
            .unwrap_err()
            .kind,
        FailureKind::ResourceLimit
    );
    assert_eq!(editor.view_state(), before);
    assert_eq!(editor.next_interaction, u64::MAX);
}
