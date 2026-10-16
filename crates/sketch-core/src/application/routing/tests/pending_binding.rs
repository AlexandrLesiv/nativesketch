use super::*;
#[test]
fn unbound_save_cannot_open_second_destination_chooser_or_cancel_active_draft() {
    let mut app = app(true, true);
    let mut queue = queue();
    save_as(&mut app, &mut queue, DestinationId(7));
    draft(&mut app, &mut queue);
    let before = app.projection();
    assert_eq!(before.save.destination(), None);
    assert!(before.save.binding_pending());
    assert_eq!(
        app.file_availability(FileIntent::Save),
        Err(ApplicationFailure::Busy)
    );
    assert!(
        app.dispatch(
            &mut queue,
            context(&app),
            ApplicationCommand::File(FileIntent::Save)
        )
        .is_err()
    );
    assert_eq!(app.projection(), before);
    assert_eq!(queue.outstanding(), 1);
    assert_eq!(bytes(&app), FIXTURE);
}
