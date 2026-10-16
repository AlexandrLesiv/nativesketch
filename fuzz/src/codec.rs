use sketch_core::Editor;

const FIXTURE: &[u8] =
    include_bytes!("../../crates/sketch-core/tests/fixtures/portable-v1.nsketch");

/// Exercise both raw framing/checksum rejection and deeper structured validation.
/// Recomputing a checksum in the harness does not disable production validation.
pub fn check(data: &[u8]) {
    if data.len() > 65_536 {
        return;
    }
    attempt(data);
    if data.len() < 24 {
        return;
    }
    let mut candidate = data.to_vec();
    let end = candidate.len() - 8;
    candidate[..8].copy_from_slice(b"NSKETCH1");
    let checksum = candidate[..end].iter().fold(0xcbf29ce484222325u64, |h, b| {
        (h ^ u64::from(*b)).wrapping_mul(0x100000001b3)
    });
    candidate[end..].copy_from_slice(&checksum.to_le_bytes());
    attempt(&candidate);
}

fn attempt(data: &[u8]) {
    let mut editor = Editor::new();
    editor.import(FIXTURE).unwrap();
    let before = editor.export().unwrap();
    if editor.import(data).is_err() {
        assert_eq!(editor.export().unwrap(), before);
        return;
    }
    let canonical = editor.export().unwrap().0;
    let mut reopened = Editor::new();
    reopened.import(&canonical).unwrap();
    assert_eq!(editor.document(), reopened.document());
    assert_eq!(reopened.export().unwrap().0, canonical);
    for command in editor.render() {
        assert!(command.points.iter().all(|value| value.is_finite()));
        assert!(command.rgba.iter().all(|value| value.is_finite()));
    }
}
