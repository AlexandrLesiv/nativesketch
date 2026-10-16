use super::*;
#[test]
fn every_effect_and_save_result_has_an_explicit_fixed_tag() {
    let mut a = app();
    let id = session(&mut a);
    let handle = a.session(id).unwrap();
    let r = file(&mut a, id, 2);
    drop(r);
    let p = a.app.projection(handle).unwrap();
    let token = p.continuation.unwrap();
    for (name, effect) in [
        ("None", None),
        (
            "ChooseDestination",
            Some(HostEffect::ChooseDestination {
                token,
                purpose: SavePurpose::SaveAs,
            }),
        ),
        ("ChooseSource", Some(HostEffect::ChooseSource(token))),
        (
            "ConfirmUnsaved",
            Some(HostEffect::ConfirmUnsaved {
                token,
                revision: Revision(0),
            }),
        ),
        ("AwaitWrites", Some(HostEffect::AwaitWrites(token))),
        ("NavigationReady", Some(HostEffect::NavigationReady(token))),
        ("CloseConfirmed", Some(HostEffect::CloseConfirmed)),
        ("Cancelled", Some(HostEffect::Cancelled)),
        (
            "Failure",
            Some(HostEffect::Failure(ApplicationFailure::Busy)),
        ),
    ] {
        let outcome = AppOutcome {
            projection: p,
            changes: StateChanges {
                document: false,
                session_replaced: false,
                settings: false,
                selection: false,
                interaction: false,
            },
            invalidation: RenderInvalidation::None,
            effect,
            save: None,
        };
        let mut reply = Reply::reserve(&a.budget, 20, 1, 4096).unwrap();
        a.encode_outcome(&mut reply, handle, outcome, 900);
        let reply = reply.finish();
        assert_eq!(
            &reply.bytes()[232..236],
            fixture(&format!("tag.effect.{name}"))
        );
    }
    let mut a = app();
    let id = session(&mut a);
    let op = queued(&mut a, id, 2, 1, 7);
    let work = take(&mut a);
    finish(&mut a, work);
    let h = a.session(id).unwrap();
    let o = a.operation(op).unwrap();
    let save = a.app.apply_ready(h, o).unwrap().save.unwrap();
    for (name, result) in [
        (
            "Written",
            SaveResult::Written(Durability::ProviderConfirmed),
        ),
        ("Uncertain", SaveResult::Uncertain(StorageFailure::Write)),
        ("Failed", SaveResult::Failed(StorageFailure::Access)),
        ("CancelledBeforeStart", SaveResult::CancelledBeforeStart),
        (
            "WrittenAfterCancellation",
            SaveResult::WrittenAfterCancellation(Durability::FileAndDirectorySynced),
        ),
        (
            "FailedAfterCancellation",
            SaveResult::FailedAfterCancellation(StorageFailure::Replace),
        ),
        (
            "UncertainAfterCancellation",
            SaveResult::UncertainAfterCancellation(StorageFailure::WorkerStopped),
        ),
        ("DispatchedUnconfirmed", SaveResult::DispatchedUnconfirmed),
        (
            "DispatchedAfterCancellation",
            SaveResult::DispatchedAfterCancellation,
        ),
    ] {
        let mut reply = Reply::reserve(&a.budget, 32, 1, 4096).unwrap();
        reply.save(Some(AppliedSave { result, ..save }));
        let reply = reply.finish();
        assert_eq!(
            &reply.bytes()[80..84],
            fixture(&format!("tag.result.{name}"))
        );
    }
}
