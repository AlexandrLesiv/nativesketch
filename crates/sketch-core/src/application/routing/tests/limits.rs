use super::*;
#[test]
fn undelivered_completion_obligations_have_bounded_admission_and_remain_observable() {
    let mut app = app(true, true);
    let mut queue = queue();
    for _ in 0..1024 {
        assert!(matches!(
            file(
                &mut app,
                &mut queue,
                FileIntent::Checkpoint(DestinationId(8))
            ),
            ApplicationEffect::SaveQueued(_)
        ));
        let _undelivered = complete(&mut queue, written());
    }
    assert_eq!(queue.outstanding(), 0);
    assert_eq!(app.projection().outstanding_saves, 1024);
    assert!(matches!(
        file(
            &mut app,
            &mut queue,
            FileIntent::Checkpoint(DestinationId(8))
        ),
        ApplicationEffect::Failure(ApplicationFailure::Busy)
    ));
    assert_eq!(app.projection().outstanding_saves, 1024);
    assert_eq!(queue.outstanding(), 0);
}
