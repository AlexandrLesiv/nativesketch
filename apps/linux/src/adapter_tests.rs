use crate::model::Model;
#[test]
fn adapter_cancel_transform_roundtrip() {
    let mut m = Model::new();
    m.layout(500, 400);
    m.down(40.0, 50.0, None, 100, 0).unwrap();
    m.motion(80.0, 80.0, None, 127).unwrap();
    m.up().unwrap();
    assert_eq!(m.editor.state().count, 1);
    assert_eq!(m.editor.document().strokes[0].samples[1].timestamp, 0.027);
    m.down(90.0, 100.0, None, 200, 0).unwrap();
    m.cancel();
    assert_eq!(m.editor.state().count, 1);
    m.editor.select_cycle(1).unwrap();
    m.editor.resize_begin().unwrap();
    m.editor.resize_update(2.0, 0.0, 0.0).unwrap();
    m.editor.resize_commit().unwrap();
    m.editor.undo().unwrap();
    let before = m.editor.export().unwrap();
    assert!(m.editor.import(b"bad").is_err());
    assert_eq!(before, m.editor.export().unwrap());
    let center = m.point(250.0, 200.0);
    m.layout(700, 600);
    assert_eq!(center, m.point(350.0, 300.0));
    let path = std::env::temp_dir().join(format!("native-sketch-gtk-{}", std::process::id()));
    std::fs::create_dir_all(&path).unwrap();
    assert!(m.editor.save(&path).is_err());
    assert!(m.editor.state().dirty);
    std::fs::remove_dir(path).unwrap();
}
