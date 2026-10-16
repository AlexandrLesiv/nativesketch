use super::*;
#[test]
fn readonly_denies_every_edit_command_and_batch_without_partial_state() {
    for export_document in [false, true] {
        let mut viewer = EditorSession::new(
            Capabilities {
                edit_document: false,
                export_document,
            },
            ToolId::Select,
            brush(6.0),
            Some(ValidatedDocument::decode(FIXTURE).unwrap()),
        )
        .unwrap();
        let before = viewer.view_state();
        use EditorCommand::*;
        let commands = [
            SetTool(ToolId::Pen),
            SetBrush(brush(9.0)),
            BeginStroke {
                pointer: PointerId(42),
                device: DeviceKind::Pen,
            },
            CommitStroke(PointerId(42)),
            BeginTransform,
            UpdateTransform {
                scale: 2.0,
                anchor_x: 0.0,
                anchor_y: 0.0,
            },
            CommitTransform,
            DeleteSelection,
            Undo,
            Redo,
            NewDocument,
            ReplaceDocument(ValidatedDocument::decode(FIXTURE).unwrap()),
        ];
        for command in commands {
            let operation = command.operation();
            let error = viewer.execute(context(&viewer), command).unwrap_err();
            assert_eq!(error.kind, FailureKind::ReadOnly);
            assert_eq!(error.operation, operation);
            assert_eq!(error.context, Some(context(&viewer)));
            assert_eq!(viewer.view_state(), before);
            assert_eq!(snapshot(&viewer), FIXTURE);
        }
        let error = viewer
            .append(
                context(&viewer),
                SampleBatch {
                    pointer: PointerId(42),
                    samples: &[Input::Actual(sample(1))],
                },
            )
            .unwrap_err();
        assert_eq!(error.kind, FailureKind::ReadOnly);
        assert_eq!(viewer.view_state(), before);
        assert_eq!(snapshot(&viewer), FIXTURE);
    }
}
#[test]
fn readonly_allows_selection_queries_and_idle_cancel_but_not_pen_constructor() {
    let mut viewer = session(false, true);
    let before = snapshot(&viewer);
    execute(&mut viewer, EditorCommand::SelectCycle(1));
    assert_ne!(viewer.view_state().document.selected, 0);
    for query in [
        EditorQuery::ViewState,
        EditorQuery::SelectedBounds,
        EditorQuery::Scene,
        EditorQuery::CommittedSnapshot,
    ] {
        viewer.query(query).unwrap();
    }
    execute(
        &mut viewer,
        EditorCommand::SelectAt {
            x: 10.0,
            y: 20.0,
            tolerance: 1.0,
        },
    );
    execute(&mut viewer, EditorCommand::ClearSelection);
    execute(&mut viewer, EditorCommand::SetTool(ToolId::Select));
    execute(&mut viewer, EditorCommand::CancelInteraction);
    assert_eq!(snapshot(&viewer), before);
    assert_eq!(
        EditorSession::new(
            Capabilities {
                edit_document: false,
                export_document: true
            },
            ToolId::Pen,
            brush(6.0),
            None
        )
        .unwrap_err()
        .kind,
        FailureKind::ReadOnly
    );
}
#[test]
fn availability_distinguishes_history_from_actions_and_readonly_queries() {
    let mut editor = session(true, true);
    begin(&mut editor);
    editor
        .append(
            context(&editor),
            SampleBatch {
                pointer: PointerId(42),
                samples: &[Input::Actual(sample(1))],
            },
        )
        .unwrap();
    execute(&mut editor, EditorCommand::CommitStroke(PointerId(42)));
    begin(&mut editor);
    let state = editor.view_state();
    assert!(state.document.can_undo);
    assert_eq!(
        state.availability(ActionId::Undo),
        Availability::Disabled(DisabledReason::InteractionActive)
    );
    assert_eq!(
        state.availability(ActionId::SetBrush),
        Availability::Enabled
    );
    assert_eq!(
        state.availability(ActionId::ChooseSelect),
        Availability::Disabled(DisabledReason::InteractionActive)
    );
    execute(&mut editor, EditorCommand::CancelInteraction);
    assert_eq!(
        editor.view_state().availability(ActionId::Undo),
        Availability::Enabled
    );
    let viewer = session(false, true).view_state();
    assert_eq!(
        viewer.availability(ActionId::Undo),
        Availability::Disabled(DisabledReason::ReadOnly)
    );
    assert_eq!(viewer.availability(ActionId::Select), Availability::Enabled);
}
