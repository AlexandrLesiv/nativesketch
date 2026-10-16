//! Deterministic generated cases: INPUT-001/002, INK-001/002/003, UNDO-001/002.
use crate::{Brush, Editor, Error, Input, Sample};

fn brush(seed: u64) -> Brush {
    Brush {
        rgba: [0.2, 0.4, 0.6, 0.5],
        width: 1.0 + seed as f64,
    }
}

fn samples(mut seed: u64) -> Vec<Sample> {
    (1..=32)
        .map(|id| {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            Sample {
                id,
                x: (seed % 10000) as f64 - 5000.0,
                y: ((seed >> 16) % 10000) as f64 - 5000.0,
                timestamp: id as f64 / 120.0,
                pressure: (id % 3 != 0).then_some((seed % 101) as f64 / 100.0),
            }
        })
        .collect()
}

fn draw(seed: u64, batch_size: usize, predictions: bool) -> Editor {
    let mut editor = Editor::new();
    let captured = brush(seed);
    editor.begin(7, 1, captured).unwrap();
    for chunk in samples(seed).chunks(batch_size) {
        let mut batch: Vec<_> = chunk.iter().copied().map(Input::Actual).collect();
        if predictions {
            batch.push(Input::Predicted(Sample {
                x: 9000.0,
                timestamp: 100.0,
                ..*chunk.last().unwrap()
            }));
        }
        editor.append(7, &batch).unwrap();
    }
    editor.commit(7).unwrap();
    assert_eq!(editor.document().strokes[0].brush, captured);
    editor
}

#[test]
fn generated_rebatching_and_predictions_preserve_document_bytes_and_one_undo() {
    for seed in 0..64 {
        let whole = draw(seed, 32, false);
        let expected = whole.export().unwrap().0;
        for batch_size in [1, 3, 7, 16, 32] {
            let mut batched = draw(seed, batch_size, true);
            assert_eq!(batched.export().unwrap().0, expected, "seed {seed}");
            assert_eq!(batched.document().strokes[0].samples, samples(seed));
            batched.undo().unwrap();
            assert_eq!(batched.state().count, 0);
            assert_eq!(batched.undo(), Err(Error::InvalidState));
            batched.redo().unwrap();
            assert_eq!(batched.export().unwrap().0, expected);
            assert_eq!(batched.redo(), Err(Error::InvalidState));
        }
    }
}

#[test]
fn generated_invalid_trailing_samples_preserve_predictions_and_cancelled_history() {
    for seed in 0..64 {
        let mut editor = draw(seed, 4, false);
        let committed = editor.export().unwrap();
        editor.begin(9, 0, brush(seed + 1)).unwrap();
        let actual = samples(seed)[0];
        editor
            .append(
                9,
                &[
                    Input::Actual(actual),
                    Input::Predicted(Sample {
                        x: 9000.0,
                        timestamp: 100.0,
                        ..actual
                    }),
                ],
            )
            .unwrap();
        let preview = format!("{:?}", editor.render());
        let valid = Sample {
            id: 2,
            timestamp: 2.0,
            ..actual
        };
        for invalid in [
            Sample {
                id: 3,
                x: f64::NAN,
                ..valid
            },
            Sample {
                id: 3,
                pressure: Some(-0.1),
                ..valid
            },
            Sample { id: 1, ..valid },
            Sample {
                id: 3,
                timestamp: 0.0,
                ..valid
            },
        ] {
            assert_eq!(
                editor.append(9, &[Input::Actual(valid), Input::Actual(invalid)]),
                Err(Error::InvalidArgument)
            );
            assert_eq!(format!("{:?}", editor.render()), preview);
            assert_eq!(editor.document().strokes.len(), 1);
        }
        assert_eq!(editor.undo(), Err(Error::InvalidState));
        editor.cancel();
        assert_eq!(editor.export().unwrap(), committed);
        editor.undo().unwrap();
        assert_eq!(editor.state().count, 0);
        assert!(!editor.state().can_undo);
    }
}

#[test]
fn successful_empty_batch_clears_only_predictions_and_brush_is_captured() {
    let mut editor = Editor::new();
    let mut settings = brush(3);
    editor.begin(1, 1, settings).unwrap();
    settings.width = 20.0;
    let actual = samples(1)[0];
    editor
        .append(
            1,
            &[
                Input::Actual(actual),
                Input::Predicted(Sample {
                    x: 9000.0,
                    timestamp: 100.0,
                    ..actual
                }),
            ],
        )
        .unwrap();
    assert!(
        editor
            .render()
            .iter()
            .any(|command| command.points[0] > 8000.0)
    );
    editor.append(1, &[]).unwrap();
    assert!(
        !editor
            .render()
            .iter()
            .any(|command| command.points[0] > 8000.0)
    );
    editor.commit(1).unwrap();
    editor.begin(2, 1, settings).unwrap();
    editor.append(2, &[Input::Actual(actual)]).unwrap();
    editor.commit(2).unwrap();
    assert_eq!(editor.document().strokes[0].brush.width, 4.0);
    assert_eq!(editor.document().strokes[1].brush.width, 20.0);
}
