use super::viewer::{candidate, source};
use super::*;
#[test]
fn viewer_route_never_grants_edit_export_or_ordinary_open_authority() {
    let mut viewer = app(false, false);
    let mut queue = queue();
    let mut prepared = candidate(source(&mut viewer, &mut queue, 1));
    run(
        &mut viewer,
        &mut queue,
        ApplicationCommand::Install(&mut prepared),
    );
    let before = viewer.projection();
    for command in [
        ApplicationCommand::File(FileIntent::ExportCopy),
        ApplicationCommand::File(FileIntent::Save),
        ApplicationCommand::File(FileIntent::SaveAs),
        ApplicationCommand::File(FileIntent::Navigate(NavigationIntent::Open)),
        ApplicationCommand::File(FileIntent::Navigate(NavigationIntent::New)),
        ApplicationCommand::Edit(EditorCommand::DeleteSelection),
        ApplicationCommand::Action(EditorAction::ChooseTool(ToolId::Pen)),
    ] {
        assert!(
            viewer
                .dispatch(&mut queue, context(&viewer), command)
                .is_err()
        );
        assert_eq!(viewer.projection(), before);
    }
    for (edit, export) in [(true, true), (true, false), (false, true)] {
        let mut app = app(edit, export);
        let before = app.projection();
        assert_eq!(
            app.dispatch(
                &mut queue,
                context(&app),
                ApplicationCommand::ViewSource {
                    source: DestinationId(99),
                    generation: 1,
                }
            )
            .unwrap_err()
            .failure,
            ApplicationFailure::CapabilityDenied
        );
        assert_eq!(app.projection(), before);
    }
}
