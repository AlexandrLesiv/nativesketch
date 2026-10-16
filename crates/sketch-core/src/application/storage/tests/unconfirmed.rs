use super::*;
// SAVE-001: dispatch is never persistence, even when targeting the bound destination.
#[test]
fn unconfirmed_copy_preserves_dirty_binding_and_saved_revision_with_cancellation() {
    for cancel in [false, true] {
        for dirty in [false, true] {
            let mut session = bound();
            if dirty {
                stroke(&mut session);
            }
            let before = session.view_state();
            let saved = session.save_state();
            let mut queue = coordinator();
            let token = request(&mut queue, &mut session, SavePurpose::ExportCopy);
            let job = queue.start_next().unwrap();
            assert_eq!(job.encoded_len().unwrap(), job.encode().unwrap().len());
            if cancel {
                queue.cancel(token).unwrap();
            }
            queue
                .finish(job.complete(WriteOutcome::DispatchedUnconfirmed))
                .unwrap();
            let completion = queue.take_completion(token).unwrap();
            assert_eq!(
                completion.result(),
                if cancel {
                    SaveResult::DispatchedAfterCancellation
                } else {
                    SaveResult::DispatchedUnconfirmed
                }
            );
            assert_eq!(
                session.acknowledge_save(completion),
                if cancel {
                    Acknowledgement::Cancelled
                } else {
                    Acknowledgement::CopyDispatched
                }
            );
            assert_eq!(session.view_state(), before);
            assert_eq!(session.save_state(), saved);
        }
    }
}
#[test]
fn coordinator_rejects_unconfirmed_non_export_receipts_without_releasing_write_barrier() {
    for purpose in [
        SavePurpose::Save,
        SavePurpose::SaveAs,
        SavePurpose::Checkpoint,
    ] {
        for cancel in [false, true] {
            let mut session = if purpose == SavePurpose::Save {
                bound()
            } else {
                session()
            };
            let mut queue = coordinator();
            let token = request(&mut queue, &mut session, purpose);
            let job = queue.start_next().unwrap();
            if cancel {
                queue.cancel(token).unwrap();
            }
            let next = queue
                .request(
                    &mut session,
                    RequestId(43),
                    SavePurpose::ExportCopy,
                    DestinationId(2),
                )
                .unwrap();
            let before = session.view_state();
            let rejection = queue
                .finish(job.complete(WriteOutcome::DispatchedUnconfirmed))
                .unwrap_err();
            assert_eq!(rejection.error(), StorageError::CapabilityDenied);
            assert!(queue.start_next().is_none());
            assert_eq!(queue.outstanding(), 2);
            assert!(queue.take_completion(token).is_err());
            assert_eq!(session.view_state(), before);
            let mut receipt = rejection.into_receipt();
            assert_eq!(
                receipt.revise_outcome(WriteOutcome::DispatchedUnconfirmed),
                Err(StorageError::CapabilityDenied)
            );
            receipt
                .revise_outcome(WriteOutcome::Failed(StorageFailure::Access))
                .unwrap();
            queue.finish(receipt).unwrap();
            let completion = queue.take_completion(token).unwrap();
            assert_eq!(
                completion.result(),
                if cancel {
                    SaveResult::FailedAfterCancellation(StorageFailure::Access)
                } else {
                    SaveResult::Failed(StorageFailure::Access)
                }
            );
            session.acknowledge_save(completion);
            let job = queue.start_next().unwrap();
            assert_eq!(job.token(), next);
            queue
                .finish(job.complete(WriteOutcome::DispatchedUnconfirmed))
                .unwrap();
            session.acknowledge_save(queue.take_completion(next).unwrap());
            assert_eq!(queue.outstanding(), 0);
        }
    }
}
#[test]
fn checked_encoding_length_tracks_multiple_committed_strokes_without_encoding_side_effects() {
    let mut session = session();
    let mut queue = coordinator();
    for _ in 0..8 {
        stroke(&mut session);
        let before = session.view_state();
        let token = request(&mut queue, &mut session, SavePurpose::ExportCopy);
        let job = queue.start_next().unwrap();
        assert_eq!(job.encoded_len().unwrap(), job.encode().unwrap().len());
        queue
            .finish(job.complete(WriteOutcome::DispatchedUnconfirmed))
            .unwrap();
        session.acknowledge_save(queue.take_completion(token).unwrap());
        assert_eq!(session.view_state(), before);
    }
}

#[test]
fn wrong_coordinator_returns_receipt_for_the_actual_owner() {
    let mut owner = coordinator();
    let mut stranger = coordinator();
    let mut session = bound();
    let token = request(&mut owner, &mut session, SavePurpose::Save);
    let job = owner.start_next().unwrap();
    let rejection = stranger
        .finish(job.complete(WriteOutcome::Failed(StorageFailure::Write)))
        .unwrap_err();
    assert_eq!(rejection.error(), StorageError::UnknownToken);
    owner.finish(rejection.into_receipt()).unwrap();
    assert_eq!(
        owner.take_completion(token).unwrap().result(),
        SaveResult::Failed(StorageFailure::Write)
    );
    assert_eq!(owner.outstanding(), 0);
}
