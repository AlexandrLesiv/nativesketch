use super::*;
#[test]
fn resize_is_preview_then_one_undo_step_including_width() {
    let mut e = Editor::new();
    draw(&mut e);
    let original = e.document().clone();
    e.select_at(10.0, 10.0, 0.0).unwrap();
    let before = e.state().bounds;
    e.resize_begin().unwrap();
    e.resize_update(2.0, 0.0, 0.0).unwrap();
    e.resize_update(3.0, 0.0, 0.0).unwrap();
    assert_eq!(e.document(), &original);
    assert_eq!(e.state().bounds.width, before.width * 3.0);
    e.resize_commit().unwrap();
    let resized = e.document().clone();
    e.undo().unwrap();
    assert_eq!(e.document(), &original);
    e.undo().unwrap();
    assert!(e.document().strokes.is_empty());
    e.redo().unwrap();
    e.redo().unwrap();
    assert_eq!(e.document(), &resized);
}
#[test]
fn cancel_and_noop_resize_preserve_redo_and_history() {
    let mut e = Editor::new();
    draw(&mut e);
    draw(&mut e);
    e.undo().unwrap();
    e.select_cycle(1).unwrap();
    let committed = e.export().unwrap();
    e.resize_begin().unwrap();
    e.resize_update(2.0, 0.0, 0.0).unwrap();
    e.cancel();
    assert_eq!(e.export().unwrap(), committed);
    assert!(e.state().can_redo);
    e.resize_begin().unwrap();
    e.resize_update(1.0, 0.0, 0.0).unwrap();
    e.resize_commit().unwrap();
    assert_eq!(e.export().unwrap(), committed);
    assert!(e.state().can_redo);
    e.redo().unwrap();
    assert_eq!(e.state().count, 2);
    e.undo().unwrap();
    e.undo().unwrap();
    assert!(!e.state().can_undo);
}
#[test]
fn selection_is_topmost_and_delete_is_undoable() {
    let mut e = Editor::new();
    draw(&mut e);
    draw(&mut e);
    e.select_at(10.0, 10.0, 0.0).unwrap();
    assert_eq!(e.state().selected, 2);
    e.delete_selected().unwrap();
    assert_eq!(e.state().count, 1);
    e.undo().unwrap();
    assert_eq!(e.document().strokes[1].id, 2);
    e.select_cycle(-1).unwrap();
    assert_eq!(e.state().selected, 2);
    e.select_cycle(-1).unwrap();
    assert_eq!(e.state().selected, 1);
}
