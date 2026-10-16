use super::file_store::{TEMP_ID, save_with};
use super::*;
use crate::{Brush, Document, Editor, Error, Input, Sample};
use std::fs;
use std::sync::atomic::Ordering;
#[test]
fn interrupted_replace_preserves_good_file() {
    let dir = std::env::temp_dir().join(format!(
        "sketch-persistence-{}",
        TEMP_ID.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join("doc.nsketch");
    fs::write(&path, b"last good").unwrap();
    let error = save_with(&Document::default(), &path, |temp| {
        assert!(temp.exists());
        Err(std::io::Error::other("injected interruption"))
    });
    assert_eq!(error, Err(Error::Io));
    assert_eq!(fs::read(&path).unwrap(), b"last good");
    assert_eq!(fs::read_dir(&dir).unwrap().count(), 1);
    fs::remove_dir_all(dir).unwrap();
}
#[test]
fn checksum_and_version_reject_corruption() {
    let b = encode(&Document::default());
    assert_eq!(decode(&b).unwrap(), Document::default());
    for i in 0..b.len() {
        let mut c = b.clone();
        c[i] ^= 1;
        assert_eq!(decode(&c), Err(Error::Format));
    }
    for i in 0..b.len() {
        assert_eq!(decode(&b[..i]), Err(Error::Format));
    }
}
#[test]
fn accepted_boundary_samples_roundtrip_from_default_editor() {
    let mut editor = Editor::default();
    let brush = Brush {
        rgba: [0.0, 0.0, 0.0, 1.0],
        width: 10000.0,
    };
    editor.begin(1, 0, brush).unwrap();
    let sample = Sample {
        id: 1,
        x: 1e9,
        y: -1e9,
        timestamp: 0.0,
        pressure: None,
    };
    editor.append(1, &[Input::Actual(sample)]).unwrap();
    editor.commit(1).unwrap();
    assert_eq!(editor.document().strokes[0].id, 1);
    assert_eq!(
        decode(&encode(editor.document())).unwrap(),
        *editor.document()
    );
    editor.select_cycle(1).unwrap();
    editor.resize_begin().unwrap();
    editor.resize_update(10.0, 0.0, 0.0).unwrap();
    editor.resize_commit().unwrap();
    assert_eq!(
        decode(&encode(editor.document())).unwrap(),
        *editor.document()
    );
    editor.begin(2, 0, brush).unwrap();
    let invalid = Sample { x: 1e12, ..sample };
    assert_eq!(
        editor.append(2, &[Input::Actual(invalid)]),
        Err(Error::InvalidArgument)
    );
}
