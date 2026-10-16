use super::*;
#[test]
fn save_roundtrip_dirty_identity_and_failed_load_save() {
    let mut e = Editor::new();
    draw(&mut e);
    e.select_cycle(1).unwrap();
    e.resize_begin().unwrap();
    e.resize_update(2.5, 5.0, 5.0).unwrap();
    e.resize_commit().unwrap();
    let dir = std::env::temp_dir().join(format!("sketch-roundtrip-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("drawing.nsketch");
    e.save(&path).unwrap();
    assert!(!e.state().dirty);
    let saved = std::fs::read(&path).unwrap();
    let mut other = Editor::new();
    other.load(&path).unwrap();
    assert_eq!(e.document(), other.document());
    assert!(!other.state().dirty);
    e.undo().unwrap();
    assert!(e.state().dirty);
    e.redo().unwrap();
    assert!(!e.state().dirty);
    e.delete_selected().unwrap();
    assert!(e.state().dirty);
    assert_eq!(e.save(&dir), Err(Error::Io));
    assert!(e.state().dirty);
    assert_eq!(std::fs::read(&path).unwrap(), saved);
    let before = e.document().clone();
    std::fs::write(dir.join("corrupt"), b"bad").unwrap();
    assert_eq!(e.load(&dir.join("corrupt")), Err(Error::Format));
    assert_eq!(e.document(), &before);
    assert_eq!(e.load(&dir.join("missing")), Err(Error::Io));
    assert_eq!(e.document(), &before);
    std::fs::remove_dir_all(&dir).unwrap();
}
#[test]
fn portable_snapshot_stale_acknowledgement_and_atomic_failure() {
    let mut e = Editor::new();
    let (empty, initial) = e.export().unwrap();
    e.clear().unwrap();
    assert_eq!(e.mark_saved(initial), Err(Error::InvalidState));
    let (_, current) = e.export().unwrap();
    e.import(&empty).unwrap();
    assert_eq!(e.mark_saved(current), Err(Error::InvalidState));
    let (before, revision) = e.export().unwrap();
    assert_eq!(e.import(b"corrupt"), Err(Error::Format));
    assert_eq!(e.export().unwrap(), (before, revision));
    e.begin(
        1,
        0,
        Brush {
            rgba: [0.0, 0.0, 0.0, 1.0],
            width: 4.0,
        },
    )
    .unwrap();
    assert_eq!(e.export(), Err(Error::InvalidState));
    assert_eq!(e.import(&empty), Err(Error::InvalidState));
    assert_eq!(e.mark_saved(revision), Err(Error::InvalidState));
    e.append(
        1,
        &[Input::Actual(Sample {
            x: 1.0,
            y: 2.0,
            timestamp: 0.0,
            pressure: None,
            id: 1,
        })],
    )
    .unwrap();
    e.commit(1).unwrap();
    assert!(e.state().dirty);
    assert_eq!(e.mark_saved(revision), Err(Error::InvalidState));
    let (_, written_revision) = e.export().unwrap();
    assert!(e.state().dirty); // cancelled/failed host write leaves dirty
    e.mark_saved(written_revision).unwrap();
    assert!(!e.state().dirty);
    e.undo().unwrap();
    assert!(e.state().dirty);
    e.redo().unwrap();
    assert!(!e.state().dirty);
    e.import(&empty).unwrap();
    assert_eq!(e.mark_saved(written_revision), Err(Error::InvalidState));
}
