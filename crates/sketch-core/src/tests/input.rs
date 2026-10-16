use super::*;
use crate::geometry::radius;
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
    assert_eq!(e.document().strokes[0].samples, vec![actual]);
    assert!(!e.render().iter().any(|c| c.points[0] > 90.0));
    assert_eq!(radius(&e.document().strokes[0], actual), 5.0);
    assert_eq!(
        radius(&e.document().strokes[0], sample(2, 0.0, Some(0.0))),
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
    assert_eq!(e.document().strokes[0].samples, vec![correction]);
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
    let trace = include_str!("../../tests/fixtures/synthetic-pressure-v1.csv");
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
    let stroke = &editor.document().strokes[0];
    assert_eq!(stroke.samples.len(), 4);
    assert_eq!(stroke.samples[0].pressure, None);
    assert_eq!(stroke.samples[3].pressure, Some(1.0));
    let original = editor.document().clone();
    editor.select_cycle(1).unwrap();
    editor.resize_begin().unwrap();
    editor.resize_update(1.0, 1e12, 1e12).unwrap();
    editor.resize_commit().unwrap();
    assert_eq!(editor.document(), &original);
    editor.undo().unwrap();
    assert_eq!(editor.state().count, 0);
    assert!(!editor.state().can_undo);
}
