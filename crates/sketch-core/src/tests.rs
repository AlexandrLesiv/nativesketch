use super::*;
fn brush() -> Brush {
    Brush {
        rgba: [0.2, 0.3, 0.4, 0.5],
        width: 10.0,
    }
}
fn sample(id: u64, x: f64, pressure: Option<f64>) -> Sample {
    Sample {
        id,
        x,
        y: 10.0,
        timestamp: id as f64 / 100.0,
        pressure,
    }
}
fn draw(e: &mut Editor) {
    e.begin(7, 1, brush()).unwrap();
    e.append(
        7,
        &[
            Input::Actual(sample(1, 0.0, None)),
            Input::Actual(sample(2, 10.0, Some(0.5))),
            Input::Actual(sample(3, 20.0, Some(1.0))),
        ],
    )
    .unwrap();
    e.commit(7).unwrap();
}
#[test]
fn replay_preserves_actual_pressure_and_discards_prediction() {
    let mut e = Editor::new();
    e.begin(7, 1, brush()).unwrap();
    let actual = sample(1, 0.0, None);
    e.append(
        7,
        &[
            Input::Actual(actual),
            Input::Predicted(sample(2, 100.0, Some(0.5))),
        ],
    )
    .unwrap();
    assert!(e.render().iter().any(|c| c.points[0] > 90.0));
    e.commit(7).unwrap();
    assert_eq!(e.document.strokes[0].samples, vec![actual]);
    assert!(!e.render().iter().any(|c| c.points[0] > 90.0));
    assert_eq!(radius(&e.document.strokes[0], actual), 5.0);
    assert_eq!(
        radius(&e.document.strokes[0], sample(2, 0.0, Some(0.0))),
        1.0
    );
}
#[test]
fn cancelled_and_stale_input_never_commits() {
    let mut e = Editor::new();
    e.begin(7, 0, brush()).unwrap();
    e.append(7, &[Input::Actual(sample(1, 0.0, None))]).unwrap();
    e.cancel();
    e.cancel();
    assert_eq!(e.commit(7), Err(Error::InvalidState));
    assert_eq!(
        e.append(7, &[Input::Actual(sample(2, 0.0, None))]),
        Err(Error::InvalidState)
    );
    assert_eq!(e.state().count, 0);
    assert!(!e.state().can_undo);
    assert!(!e.state().dirty);
}
#[test]
fn corrections_and_invalid_batches_are_atomic() {
    let mut e = Editor::new();
    e.begin(7, 1, brush()).unwrap();
    e.append(7, &[Input::Actual(sample(1, 0.0, None))]).unwrap();
    let correction = sample(1, 4.0, Some(0.8));
    e.append(7, &[Input::Correction(correction)]).unwrap();
    assert_eq!(
        e.append(
            7,
            &[
                Input::Actual(sample(2, 10.0, None)),
                Input::Actual(sample(3, f64::NAN, None))
            ]
        ),
        Err(Error::InvalidArgument)
    );
    e.commit(7).unwrap();
    assert_eq!(e.document.strokes[0].samples, vec![correction]);
}
#[test]
fn resize_is_preview_then_one_undo_step_including_width() {
    let mut e = Editor::new();
    draw(&mut e);
    let original = e.document.clone();
    e.select_at(10.0, 10.0, 0.0).unwrap();
    let before = e.state().bounds;
    e.resize_begin().unwrap();
    e.resize_update(2.0, 0.0, 0.0).unwrap();
    e.resize_update(3.0, 0.0, 0.0).unwrap();
    assert_eq!(e.document, original);
    assert_eq!(e.state().bounds.width, before.width * 3.0);
    e.resize_commit().unwrap();
    let resized = e.document.clone();
    e.undo().unwrap();
    assert_eq!(e.document, original);
    e.undo().unwrap();
    assert!(e.document.strokes.is_empty());
    e.redo().unwrap();
    e.redo().unwrap();
    assert_eq!(e.document, resized);
}
#[test]
fn cancel_and_noop_resize_preserve_redo_and_history() {
    let mut e = Editor::new();
    draw(&mut e);
    draw(&mut e);
    e.undo().unwrap();
    e.select_cycle(1).unwrap();
    let history = e.undo.len();
    e.resize_begin().unwrap();
    e.resize_update(2.0, 0.0, 0.0).unwrap();
    e.cancel();
    assert_eq!(e.undo.len(), history);
    assert!(e.state().can_redo);
    e.resize_begin().unwrap();
    e.resize_update(1.0, 0.0, 0.0).unwrap();
    e.resize_commit().unwrap();
    assert_eq!(e.undo.len(), history);
    assert!(e.state().can_redo);
    e.redo().unwrap();
    assert_eq!(e.state().count, 2);
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
    assert_eq!(e.document.strokes[1].id, 2);
    e.select_cycle(-1).unwrap();
    assert_eq!(e.state().selected, 2);
    e.select_cycle(-1).unwrap();
    assert_eq!(e.state().selected, 1);
}
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
    assert_eq!(e.document, other.document);
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
    let before = e.document.clone();
    std::fs::write(dir.join("corrupt"), b"bad").unwrap();
    assert_eq!(e.load(&dir.join("corrupt")), Err(Error::Format));
    assert_eq!(e.document, before);
    assert_eq!(e.load(&dir.join("missing")), Err(Error::Io));
    assert_eq!(e.document, before);
    std::fs::remove_dir_all(&dir).unwrap();
}
#[test]
fn rejects_invalid_pressure_transform_and_gesture_conflicts() {
    let mut e = Editor::new();
    draw(&mut e);
    e.select_cycle(1).unwrap();
    e.resize_begin().unwrap();
    assert_eq!(
        e.resize_update(f64::INFINITY, 0.0, 0.0),
        Err(Error::InvalidArgument)
    );
    assert_eq!(e.resize_update(0.0, 0.0, 0.0), Err(Error::InvalidArgument));
    assert_eq!(e.undo(), Err(Error::InvalidState));
    e.cancel();
    e.begin(7, 1, brush()).unwrap();
    assert_eq!(
        e.append(7, &[Input::Actual(sample(1, 0.0, Some(1.1)))]),
        Err(Error::InvalidArgument)
    );
    assert_eq!(e.commit(8), Err(Error::InvalidState));
}

#[test]
fn replay_named_synthetic_trace_and_save_only_actual_samples() {
    let mut editor = Editor::new();
    editor.begin(12, 1, brush()).unwrap();
    let trace = include_str!("../tests/fixtures/synthetic-pressure-v1.csv");
    let mut inputs = Vec::new();
    for line in trace.lines().filter(|l| !l.starts_with('#')) {
        let c: Vec<_> = line.split(',').collect();
        let sample = Sample {
            id: c[1].parse().unwrap(),
            x: c[2].parse().unwrap(),
            y: c[3].parse().unwrap(),
            timestamp: c[4].parse().unwrap(),
            pressure: if c[5] == "none" {
                None
            } else {
                Some(c[5].parse().unwrap())
            },
        };
        inputs.push(if c[0] == "actual" {
            Input::Actual(sample)
        } else {
            Input::Predicted(sample)
        });
    }
    editor.append(12, &inputs).unwrap();
    editor.commit(12).unwrap();
    let stroke = &editor.document.strokes[0];
    assert_eq!(stroke.samples.len(), 4);
    assert_eq!(stroke.samples[0].pressure, None);
    assert_eq!(stroke.samples[3].pressure, Some(1.0));
    let original = editor.document.clone();
    editor.select_cycle(1).unwrap();
    editor.resize_begin().unwrap();
    editor.resize_update(1.0, 1e12, 1e12).unwrap();
    editor.resize_commit().unwrap();
    assert_eq!(editor.document, original);
    assert_eq!(editor.undo.len(), 1);
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
