use super::*;
#[test]
fn older_save_keeps_new_edits_dirty_and_undo_finds_actual_saved_revision() {
    let mut session = bound();
    let mut queue = coordinator();
    request(&mut queue, &mut session, SavePurpose::Save);
    stroke(&mut session);
    assert_eq!(
        session.acknowledge_save(written(&mut queue)),
        Acknowledgement::SavedOlder
    );
    assert!(session.view_state().document.dirty);
    execute(&mut session, EditorCommand::Undo);
    assert!(!session.view_state().document.dirty);
}
#[test]
fn save_b_finishing_after_undo_to_saved_a_makes_a_dirty_and_redo_b_clean() {
    let mut session = bound();
    let mut queue = coordinator();
    stroke(&mut session);
    request(&mut queue, &mut session, SavePurpose::Save);
    execute(&mut session, EditorCommand::Undo);
    assert!(!session.view_state().document.dirty);
    assert_eq!(
        session.acknowledge_save(written(&mut queue)),
        Acknowledgement::SavedOlder
    );
    assert!(session.view_state().document.dirty);
    execute(&mut session, EditorCommand::Redo);
    assert!(!session.view_state().document.dirty);
}
#[test]
fn replacement_rejects_completion_and_resets_binding() {
    let mut session = bound();
    let mut queue = coordinator();
    request(&mut queue, &mut session, SavePurpose::Save);
    execute(&mut session, EditorCommand::NewDocument);
    assert_eq!(
        session.acknowledge_save(written(&mut queue)),
        Acknowledgement::StaleSession
    );
    assert_eq!(session.save_state().destination(), None);
}
#[test]
fn save_as_adopts_only_success_and_obsolete_binding_cannot_restore_old_destination() {
    let mut session = bound();
    let mut queue = coordinator();
    request(&mut queue, &mut session, SavePurpose::Save);
    queue
        .request(
            &mut session,
            RequestId(43),
            SavePurpose::SaveAs,
            DestinationId(2),
        )
        .unwrap();
    let old = written(&mut queue);
    let new = written(&mut queue);
    assert_eq!(session.save_state().destination(), Some(DESTINATION));
    assert_eq!(session.acknowledge_save(new), Acknowledgement::SavedCurrent);
    assert_eq!(session.acknowledge_save(old), Acknowledgement::StaleBinding);
    assert_eq!(session.save_state().destination(), Some(DestinationId(2)));
}
#[test]
fn checkpoints_and_export_copies_never_clear_dirty_edits() {
    let mut session = session();
    let mut queue = coordinator();
    stroke(&mut session);
    for (purpose, expected) in [
        (SavePurpose::Checkpoint, Acknowledgement::CheckpointWritten),
        (SavePurpose::ExportCopy, Acknowledgement::CopyWritten),
    ] {
        request(&mut queue, &mut session, purpose);
        assert_eq!(session.acknowledge_save(written(&mut queue)), expected);
        assert!(session.view_state().document.dirty);
    }
}
#[test]
fn reordered_acknowledgements_cannot_revert_saved_revision() {
    let mut session = bound();
    let mut queue = coordinator();
    request(&mut queue, &mut session, SavePurpose::Save);
    stroke(&mut session);
    request(&mut queue, &mut session, SavePurpose::Save);
    let old = written(&mut queue);
    let new = written(&mut queue);
    assert_eq!(session.acknowledge_save(new), Acknowledgement::SavedCurrent);
    assert_eq!(session.acknowledge_save(old), Acknowledgement::Superseded);
    assert!(!session.view_state().document.dirty);
}
#[test]
fn obsolete_binding_write_to_current_destination_invalidates_clean_state() {
    let mut session = bound();
    let mut queue = coordinator();
    request(&mut queue, &mut session, SavePurpose::SaveAs);
    stroke(&mut session);
    request(&mut queue, &mut session, SavePurpose::Save);
    execute(&mut session, EditorCommand::Undo);
    assert_eq!(
        session.acknowledge_save(written(&mut queue)),
        Acknowledgement::SavedCurrent
    );
    assert!(!session.view_state().document.dirty);
    assert_eq!(
        session.acknowledge_save(written(&mut queue)),
        Acknowledgement::StaleBinding
    );
    assert!(session.view_state().document.dirty);
}
