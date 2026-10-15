//! Task 015 fixed-byte baseline: INK-001/002, INPUT-001/002, UNDO-001/002, LOAD-001.
use sketch_core::{Brush, Editor, Error, Input, Sample};

const GOLDEN: &[u8] = include_bytes!("fixtures/portable-v1.nsketch");

fn sample(id: u64, x: f64, y: f64, pressure: f64) -> Sample {
    Sample {
        id,
        x,
        y,
        timestamp: id as f64,
        pressure: Some(pressure),
    }
}

fn begin(editor: &mut Editor) {
    editor
        .begin(
            42,
            1,
            Brush {
                rgba: [0.2, 0.3, 0.4, 0.5],
                width: 6.0,
            },
        )
        .unwrap();
}

#[test]
fn committed_predictions_match_fixed_document_bytes() {
    let mut editor = Editor::new();
    begin(&mut editor);
    editor
        .append(
            42,
            &[
                Input::Actual(sample(1, 10.0, 20.0, 0.5)),
                Input::Actual(sample(2, 30.0, 40.0, 0.7)),
                Input::Predicted(sample(3, 80.0, 90.0, 0.9)),
            ],
        )
        .unwrap();
    assert_eq!(editor.export(), Err(Error::InvalidState));
    assert_eq!(editor.undo(), Err(Error::InvalidState));
    editor.commit(42).unwrap();
    assert_eq!(editor.export().unwrap().0, GOLDEN);
    editor.undo().unwrap();
    assert!(editor.document().strokes.is_empty());
    assert!(!editor.state().can_undo);
    editor.redo().unwrap();
    assert_eq!(editor.export().unwrap().0, GOLDEN);
}

#[test]
fn frozen_document_roundtrips_and_invalid_import_retains_history() {
    let mut editor = Editor::new();
    editor.import(GOLDEN).unwrap();
    assert_eq!(editor.export().unwrap().0, GOLDEN);
    let stroke = &editor.document().strokes[0];
    assert_eq!((stroke.id, stroke.pointer_id, stroke.device), (1, 42, 1));
    assert_eq!(stroke.samples.len(), 2);
    assert_eq!(stroke.samples[1], sample(2, 30.0, 40.0, 0.7));
    assert!(!editor.state().dirty);
    assert!(!editor.state().can_undo);
    editor.select_cycle(1).unwrap();
    editor.delete_selected().unwrap();
    let before = editor.export().unwrap();
    for length in 0..GOLDEN.len() {
        assert_eq!(editor.import(&GOLDEN[..length]), Err(Error::Format));
        assert_eq!(editor.export().unwrap(), before);
        assert!(editor.state().can_undo);
    }
    editor.undo().unwrap();
    assert_eq!(editor.export().unwrap().0, GOLDEN);
}

#[test]
fn invalid_batch_and_cancel_preserve_frozen_document() {
    let mut editor = Editor::new();
    editor.import(GOLDEN).unwrap();
    begin(&mut editor);
    editor
        .append(42, &[Input::Actual(sample(1, 0.0, 0.0, 0.5))])
        .unwrap();
    let before = editor.render();
    let bad = [
        Input::Actual(sample(2, 10.0, 10.0, 0.5)),
        Input::Actual(sample(3, f64::NAN, 10.0, 0.5)),
    ];
    assert_eq!(editor.append(42, &bad), Err(Error::InvalidArgument));
    let after = editor.render();
    assert_eq!(before.len(), after.len());
    for (a, b) in before.iter().zip(after) {
        assert_eq!(
            (a.stroke_id, a.verb, a.points),
            (b.stroke_id, b.verb, b.points)
        );
    }
    editor.cancel();
    assert_eq!(editor.export().unwrap().0, GOLDEN);
    assert!(!editor.state().can_undo);
}

#[test]
fn correction_requires_an_existing_id_and_identical_timestamp() {
    let mut editor = Editor::new();
    begin(&mut editor);
    let actual = sample(1, 10.0, 20.0, 0.5);
    let correction = Input::Correction(Sample { x: 12.0, ..actual });
    assert_eq!(
        editor.append(42, &[Input::Actual(actual), correction]),
        Err(Error::InvalidArgument)
    );
    editor.append(42, &[Input::Actual(actual)]).unwrap();
    assert_eq!(
        editor.append(
            42,
            &[Input::Correction(Sample {
                timestamp: 2.0,
                ..actual
            })]
        ),
        Err(Error::InvalidArgument)
    );
    editor.append(42, &[correction]).unwrap();
    editor.commit(42).unwrap();
    assert_eq!(editor.document().strokes[0].samples[0].x, 12.0);
}
