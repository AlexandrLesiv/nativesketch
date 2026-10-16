use super::*;
#[test]
fn stable_tokens_survive_batches_but_reused_pointer_rejects_old_transaction() {
    let mut editor = session(true, false);
    let old = begin(&mut editor);
    let generation = editor.view_state().scene_generation;
    for id in 1..=3 {
        editor
            .append(
                old,
                SampleBatch {
                    pointer: PointerId(42),
                    samples: &[Input::Actual(sample(id))],
                },
            )
            .unwrap();
        assert_eq!(context(&editor).interaction, old.interaction);
    }
    assert!(editor.view_state().scene_generation.get() > generation.get());
    execute(&mut editor, EditorCommand::CancelInteraction);
    let current = begin(&mut editor);
    assert_ne!(current.interaction, old.interaction);
    let before = editor.view_state();
    for command in [
        EditorCommand::CommitStroke(PointerId(42)),
        EditorCommand::CancelInteraction,
    ] {
        assert_eq!(
            editor.execute(old, command).unwrap_err().kind,
            FailureKind::StaleInteraction
        );
    }
    assert_eq!(
        editor
            .append(
                old,
                SampleBatch {
                    pointer: PointerId(42),
                    samples: &[Input::Actual(sample(1))]
                }
            )
            .unwrap_err()
            .kind,
        FailureKind::StaleInteraction
    );
    let missing = CommandContext {
        interaction: None,
        ..current
    };
    assert_eq!(
        editor
            .append(
                missing,
                SampleBatch {
                    pointer: PointerId(42),
                    samples: &[]
                }
            )
            .unwrap_err()
            .kind,
        FailureKind::StaleInteraction
    );
    assert_eq!(
        editor
            .execute(missing, EditorCommand::CancelInteraction)
            .unwrap_err()
            .kind,
        FailureKind::StaleInteraction
    );
    assert_eq!(editor.view_state(), before);
    editor
        .execute(current, EditorCommand::CancelInteraction)
        .unwrap();
    assert_eq!(
        editor
            .execute(current, EditorCommand::CancelInteraction)
            .unwrap_err()
            .kind,
        FailureKind::StaleInteraction
    );
}
#[test]
fn invalid_batch_leaves_preview_summary_and_generation_unchanged() {
    let mut editor = session(true, true);
    let context = begin(&mut editor);
    editor
        .append(
            context,
            SampleBatch {
                pointer: PointerId(42),
                samples: &[
                    Input::Actual(sample(1)),
                    Input::Predicted(Sample {
                        x: 9000.0,
                        ..sample(3)
                    }),
                ],
            },
        )
        .unwrap();
    let before = editor.view_state();
    let preview = scene(&editor);
    let committed = snapshot(&editor);
    assert_eq!(
        editor
            .append(
                context,
                SampleBatch {
                    pointer: PointerId(42),
                    samples: &[
                        Input::Actual(sample(2)),
                        Input::Actual(Sample {
                            x: f64::NAN,
                            ..sample(3)
                        })
                    ]
                }
            )
            .unwrap_err()
            .kind,
        FailureKind::InvalidValue
    );
    assert_eq!(editor.view_state(), before);
    assert_eq!(scene(&editor), preview);
    assert_eq!(snapshot(&editor), committed);
    let error = editor
        .execute(context, EditorCommand::SetTool(ToolId::Select))
        .unwrap_err();
    assert_eq!(error.kind, FailureKind::InvalidTransition);
    assert_eq!(editor.view_state(), before);
}
#[test]
fn transform_requires_current_token_and_commits_one_history_operation() {
    let mut editor = session(true, true);
    execute(&mut editor, EditorCommand::SetTool(ToolId::Select));
    execute(&mut editor, EditorCommand::SelectCycle(1));
    let before = snapshot(&editor);
    execute(&mut editor, EditorCommand::BeginTransform);
    let token = context(&editor);
    let missing = CommandContext {
        interaction: None,
        ..token
    };
    assert_eq!(
        editor
            .execute(
                missing,
                EditorCommand::UpdateTransform {
                    scale: 2.0,
                    anchor_x: 0.0,
                    anchor_y: 0.0
                }
            )
            .unwrap_err()
            .kind,
        FailureKind::StaleInteraction
    );
    assert_eq!(
        editor
            .execute(missing, EditorCommand::CommitTransform)
            .unwrap_err()
            .kind,
        FailureKind::StaleInteraction
    );
    for scale in [1.5, 2.0, 3.0] {
        editor
            .execute(
                token,
                EditorCommand::UpdateTransform {
                    scale,
                    anchor_x: 0.0,
                    anchor_y: 0.0,
                },
            )
            .unwrap();
    }
    assert_eq!(snapshot(&editor), before);
    editor
        .execute(token, EditorCommand::CommitTransform)
        .unwrap();
    assert_ne!(snapshot(&editor), before);
    execute(&mut editor, EditorCommand::Undo);
    assert_eq!(snapshot(&editor), before);
    assert!(!editor.view_state().document.can_undo);
}
