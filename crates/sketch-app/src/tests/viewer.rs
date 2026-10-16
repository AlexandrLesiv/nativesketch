use super::*;
// VIEW-001: facade callers cannot bypass core read-only capabilities through another intent.
#[test]
fn viewer_rejects_document_mutation_and_export_through_every_intent_family() {
    let (mut app, handle) = app(AppLimits::default(), false);
    let commands = [
        EditorCommand::SetTool(ToolId::Pen),
        EditorCommand::SetBrush(config(true).brush),
        EditorCommand::BeginStroke {
            pointer: PointerId(1),
            device: DeviceKind::Pen,
        },
        EditorCommand::CommitStroke(PointerId(1)),
        EditorCommand::BeginTransform,
        EditorCommand::UpdateTransform {
            scale: 2.0,
            anchor_x: 0.0,
            anchor_y: 0.0,
        },
        EditorCommand::CommitTransform,
        EditorCommand::DeleteSelection,
        EditorCommand::Undo,
        EditorCommand::Redo,
        EditorCommand::NewDocument,
        EditorCommand::ReplaceDocument(ValidatedDocument::decode(FIXTURE).unwrap()),
    ];
    let before = app.projection(handle).unwrap();
    for command in commands {
        assert!(
            app.dispatch(handle, ctx(&app, handle), Intent::Edit(command))
                .is_err()
        );
        assert_eq!(app.projection(handle).unwrap(), before);
    }
    for action in [
        EditorAction::ChooseTool(ToolId::Pen),
        EditorAction::DeleteSelection,
        EditorAction::Undo,
        EditorAction::Redo,
        EditorAction::ScaleSelection {
            factor: ScaleFactor::try_from(2.0).unwrap(),
            anchor: ScaleAnchor::SelectionCenter,
        },
    ] {
        assert!(
            app.dispatch(handle, ctx(&app, handle), Intent::Action(action))
                .is_err()
        );
        assert_eq!(app.projection(handle).unwrap(), before);
    }
    for file in [
        FileIntent::Save,
        FileIntent::SaveAs,
        FileIntent::ExportCopy,
        FileIntent::Checkpoint(DestinationId(1)),
        FileIntent::Navigate(NavigationIntent::New),
        FileIntent::Navigate(NavigationIntent::Open),
    ] {
        assert!(
            app.dispatch(handle, ctx(&app, handle), Intent::File(file))
                .is_err()
        );
        assert_eq!(app.projection(handle).unwrap(), before);
    }
    let samples = [Input::Actual(Sample {
        id: 1,
        x: 0.0,
        y: 0.0,
        timestamp: 1.0,
        pressure: None,
    })];
    assert!(
        app.dispatch(
            handle,
            ctx(&app, handle),
            Intent::Input(SampleBatch {
                pointer: PointerId(1),
                samples: &samples
            })
        )
        .is_err()
    );
    assert_eq!(app.projection(handle).unwrap(), before);
}

#[test]
fn unconfirmed_save_as_rejection_does_not_accept_resource_or_binding() {
    let (mut app, handle) = app(AppLimits::default(), true);
    let outcome = run(&mut app, handle, Intent::File(FileIntent::SaveAs));
    let Some(HostEffect::ChooseDestination { token, .. }) = outcome.effect else {
        panic!()
    };
    let before = app.projection(handle).unwrap();
    let mut download = resource(1);
    download.access.confirmed_write = false;
    assert!(matches!(
        app.supply_resource(handle, ctx(&app, handle), token, download),
        Err(AppError::CapabilityDenied)
    ));
    assert_eq!(app.projection(handle).unwrap(), before);
    assert_eq!(app.stats().resource_leases, 0);
    assert!(app.take_release().is_none());
}
