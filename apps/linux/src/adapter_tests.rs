use crate::test_support::Fixture;
use sketch_core::application::*;
#[test]
fn adapter_cancel_transform_roundtrip() {
    let mut f = Fixture::new();
    f.model.layout(500, 400);
    f.stroke();
    assert_eq!(f.model.owner.state().count, 1);
    let before = f.snapshot();
    // Portable-v1 sample two timestamp: header 16 + stroke 96 + sample 48 + id/x/y 24.
    assert_eq!(
        f64::from_le_bytes(before[184..192].try_into().unwrap()),
        0.027
    );
    f.model.down(90.0, 100.0, None, 200, 0).unwrap();
    f.model.cancel();
    assert_eq!(f.model.owner.state().count, 1);
    f.action(EditorAction::CycleSelection(CycleDirection::Next));
    f.action(EditorAction::ScaleSelection {
        factor: 2.0.try_into().unwrap(),
        anchor: ScaleAnchor::SelectionCenter,
    });
    f.action(EditorAction::Undo);
    assert_eq!(f.snapshot(), before);
    let bad = f.directory.join("bad.nsketch");
    std::fs::write(&bad, b"bad").unwrap();
    f.open(bad);
    assert!(matches!(f.effect(), sketch_app::HostEffect::Failure(_)));
    assert_eq!(f.snapshot(), before);
    let center = f.model.point(250.0, 200.0);
    f.model.layout(700, 600);
    assert_eq!(center, f.model.point(350.0, 300.0));
    f.file(routing::FileIntent::SaveAs);
    f.choose(f.directory.clone());
    f.drain();
    assert!(matches!(f.effect(), sketch_app::HostEffect::Failure(_)));
    assert!(f.model.owner.state().dirty);
}
#[test]
fn typed_button_and_shortcut_intents_cancel_drafts_and_undo_once() {
    for route in ["button", "canvas shortcut"] {
        let mut f = Fixture::new();
        f.stroke();
        f.model.down(90.0, 90.0, None, 200, 0).unwrap();
        let intent = if route == "button" {
            sketch_app::Intent::Action(EditorAction::Undo)
        } else {
            crate::shortcut_intent::translate(gtk::gdk::Key::z, true, false).unwrap()
        };
        f.model.owner.dispatch(intent).unwrap();
        assert_eq!(f.model.owner.state().count, 0);
        assert_eq!(f.model.owner.state().interaction, 0);
        f.action(EditorAction::Redo);
        assert_eq!(f.model.owner.state().count, 1);
        f.model.zoom_to(100.0);
        assert_eq!(f.model.zoom, 32.0);
        f.model.zoom_to(0.0);
        assert_eq!(f.model.zoom, 0.05);
    }
}
