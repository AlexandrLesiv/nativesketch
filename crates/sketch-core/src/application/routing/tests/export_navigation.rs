use super::*;
// SAVE-001: settling export may release a barrier, but never satisfies a Save choice.
#[test]
fn unconfirmed_export_settles_navigation_barrier_but_still_prompts_for_unsaved_edits() {
    let mut app = app(true, true);
    let mut queue = queue();
    stroke(&mut app, &mut queue);
    let ApplicationEffect::ChooseDestination { token, .. } =
        file(&mut app, &mut queue, FileIntent::ExportCopy)
    else {
        panic!()
    };
    reply(
        &mut app,
        &mut queue,
        token,
        EffectDecision::Destination(DestinationId(9)),
    );
    assert!(matches!(
        file(
            &mut app,
            &mut queue,
            FileIntent::Navigate(NavigationIntent::New)
        ),
        ApplicationEffect::AwaitWrites(_)
    ));
    let mut completion = Some(complete(&mut queue, WriteOutcome::DispatchedUnconfirmed));
    let outcome = run(
        &mut app,
        &mut queue,
        ApplicationCommand::CompleteSave(&mut completion),
    );
    assert_eq!(
        outcome.save.unwrap().acknowledgement,
        Acknowledgement::CopyDispatched
    );
    assert!(matches!(
        outcome.effect,
        Some(ApplicationEffect::ConfirmUnsaved { .. })
    ));
    assert!(outcome.projection.editor.document.dirty);
    assert_eq!(outcome.projection.save.destination(), None);
}
