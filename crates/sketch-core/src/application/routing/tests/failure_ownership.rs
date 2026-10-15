use super::*;
#[test]
fn rejected_replacement_failure_is_retained_for_correct_owner_and_consumed_once() {
    let mut owner = app(true, true);
    let mut other = app(true, true);
    let mut queue = queue();
    let ApplicationEffect::PrepareReplacement(ticket) = file(
        &mut owner,
        &mut queue,
        FileIntent::Navigate(NavigationIntent::New),
    ) else {
        panic!()
    };
    file(
        &mut other,
        &mut queue,
        FileIntent::Navigate(NavigationIntent::New),
    );
    let before_owner = owner.projection();
    let before_other = other.projection();
    let mut failure = Some(ticket.fail(StorageFailure::Access));
    // A stale/wrong context must leave unique terminal-event ownership available for retry.
    assert!(
        owner
            .dispatch(
                &mut queue,
                context(&other),
                ApplicationCommand::ReplacementFailed(&mut failure)
            )
            .is_err()
    );
    assert!(failure.is_some());
    assert_eq!(owner.projection(), before_owner);
    // A valid context for another session cannot consume this owner's issued event either.
    assert!(
        other
            .dispatch(
                &mut queue,
                context(&other),
                ApplicationCommand::ReplacementFailed(&mut failure)
            )
            .is_err()
    );
    assert!(failure.is_some());
    assert_eq!(other.projection(), before_other);
    let outcome = run(
        &mut owner,
        &mut queue,
        ApplicationCommand::ReplacementFailed(&mut failure),
    );
    assert!(failure.is_none());
    assert!(matches!(
        outcome.effect,
        Some(ApplicationEffect::Failure(ApplicationFailure::Write(
            StorageFailure::Access
        )))
    ));
    assert_eq!(owner.projection().file, FilePhase::Idle);
    assert_eq!(bytes(&owner), FIXTURE);
    let after = owner.projection();
    assert!(
        owner
            .dispatch(
                &mut queue,
                context(&owner),
                ApplicationCommand::ReplacementFailed(&mut failure)
            )
            .is_err()
    );
    assert_eq!(owner.projection(), after);
}
