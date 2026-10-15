use sketch_services::recovery::CheckpointSchedule;
#[test]
fn debounce_max_schedule_input_deferral_and_explicit_save_priority_are_deterministic() {
    let mut schedule = CheckpointSchedule::default();
    schedule.changed(1, 1, 0);
    assert!(schedule.due(1999, false, false).is_none());
    assert!(schedule.due(2000, true, false).is_none());
    assert!(schedule.due(2000, false, true).is_none());
    let first = schedule.due(2000, false, false).unwrap();
    schedule.complete(first, false);
    for second in 1..=30 {
        schedule.changed(1, second, second * 1000);
        assert!(schedule.due(second * 1000, false, false).is_none());
    }
    assert!(schedule.due(31_000, true, false).is_none());
    assert_eq!(schedule.due(31_000, false, false).unwrap().revision, 30);
}
#[test]
fn only_latest_pending_survives_failure_requires_retry_and_stale_completion_is_rejected() {
    let mut schedule = CheckpointSchedule::default();
    schedule.changed(1, 1, 0);
    let first = schedule.due(2000, false, false).unwrap();
    for revision in 2..100 {
        schedule.changed(1, revision, revision * 1000);
    }
    assert!(schedule.due(200_000, false, false).is_none());
    assert!(schedule.complete(first, true));
    assert!(schedule.due(200_000, false, false).is_none());
    schedule.resume();
    let latest = schedule.due(200_000, false, false).unwrap();
    assert_eq!(latest.revision, 99);
    schedule.replace_session();
    schedule.changed(2, 1, 200_000);
    assert!(!schedule.complete(latest, false));
    assert_eq!(schedule.due(202_000, false, false).unwrap().session, 2);
}
