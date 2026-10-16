use crate::{shortcut_intent, test_support::Fixture};
use sketch_app::{HostEffect, Intent};
use sketch_core::application::routing::*;

#[test]
fn save_shortcut_dispatches_shared_save_intent() {
    assert!(matches!(
        shortcut_intent::translate(gtk::gdk::Key::s, true, false),
        Some(Intent::File(FileIntent::Save))
    ));
}

#[test]
fn direct_and_guarded_save_choose_again_without_overwriting_bound_file() {
    for navigation in [
        None,
        Some(NavigationIntent::New),
        Some(NavigationIntent::Open),
        Some(NavigationIntent::Close),
    ] {
        let mut f = Fixture::new();
        f.stroke();
        f.file(FileIntent::Save);
        let path = f.directory.join("original.nsketch");
        f.choose(path.clone());
        f.drain();
        let before = std::fs::read(&path).unwrap();
        assert!(!f.model.owner.state().dirty);
        f.stroke();
        match navigation {
            Some(destination) => {
                f.file(FileIntent::Navigate(destination));
                let HostEffect::ConfirmUnsaved { token, .. } = f.effect() else {
                    panic!("expected shared unsaved-navigation guard")
                };
                f.reply(token, EffectDecision::Unsaved(UnsavedChoice::Save));
            }
            None => f.file(FileIntent::Save),
        }
        let HostEffect::ChooseDestination { token, .. } = f.effect() else {
            panic!("Save must choose again after a prior confirmed write")
        };
        assert_eq!(f.model.owner.app.stats().operations, 0);
        assert_eq!(std::fs::read(&path).unwrap(), before);
        assert!(f.model.owner.state().dirty);
        f.reply(token, EffectDecision::Cancel);
        assert!(matches!(f.effect(), HostEffect::Cancelled));
        assert!(f.model.owner.effects.is_empty());
        assert!(f.model.owner.state().dirty);
        assert_eq!(f.model.owner.state().count, 2);
        assert_eq!(std::fs::read(&path).unwrap(), before);
    }
}
