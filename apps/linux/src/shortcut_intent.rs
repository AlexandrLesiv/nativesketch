use gtk::gdk;
use sketch_app::Intent;
use sketch_core::application::{routing::*, *};
/// Canvas-only key translation. GTK native editable controls never call this route.
pub(crate) fn translate(key: gdk::Key, ctrl: bool, shift: bool) -> Option<Intent<'static>> {
    let file = if ctrl {
        match key {
            gdk::Key::s => Some(FileIntent::Save),
            gdk::Key::o => Some(FileIntent::Navigate(NavigationIntent::Open)),
            gdk::Key::n => Some(FileIntent::Navigate(NavigationIntent::New)),
            _ => None,
        }
    } else {
        None
    };
    if let Some(file) = file {
        return Some(Intent::File(file));
    }
    if key == gdk::Key::Escape {
        return Some(Intent::Edit(EditorCommand::CancelInteraction));
    }
    let action = match key {
        gdk::Key::z | gdk::Key::Z if ctrl => {
            if shift {
                EditorAction::Redo
            } else {
                EditorAction::Undo
            }
        }
        gdk::Key::Delete | gdk::Key::BackSpace => EditorAction::DeleteSelection,
        gdk::Key::p if !ctrl => EditorAction::ChooseTool(ToolId::Pen),
        gdk::Key::v if !ctrl => EditorAction::ChooseTool(ToolId::Select),
        _ => return None,
    };
    Some(Intent::Action(action))
}
