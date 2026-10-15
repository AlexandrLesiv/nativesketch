use super::support::*;
use sketch_core::application::{routing::*, *};
use sketch_services::{native_runtime::*, recovery::*, storage::NoFaults};
use std::fs;
pub fn configured() -> (
    NativeRuntime,
    u64,
    std::path::PathBuf,
    RecoveryStore,
    RecoveryId,
) {
    let (empty, w, root) = fixture();
    let QueryResult::Document(snapshot) = empty
        .session(w)
        .unwrap()
        .app
        .query(EditorQuery::CommittedSnapshot)
        .unwrap()
    else {
        panic!()
    };
    let bytes = snapshot.encode().unwrap();
    close(empty, w);
    let recovery_root = root.join("recovery");
    fs::create_dir(&recovery_root).unwrap();
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(&recovery_root, fs::Permissions::from_mode(0o700)).unwrap();
    let store =
        RecoveryStore::open(recovery_root.join("sessions"), RecoveryLimits::default()).unwrap();
    let identity = RecoveryId::from_bytes([77; 16]).unwrap();
    store.allocate(identity).unwrap();
    store
        .publish(
            RecoveryEnvelope::new(
                CheckpointMetadata {
                    identity,
                    generation: 1,
                    revision: 0,
                    created_seconds: 1,
                    display_name: "recover-me".into(),
                },
                bytes,
            )
            .unwrap(),
            &NoFaults,
        )
        .unwrap();
    let mut runtime = NativeRuntime::new(RuntimeConfig {
        recovery_root: Some(recovery_root),
        ..Default::default()
    })
    .unwrap();
    let window = runtime
        .create(
            Capabilities {
                edit_document: true,
                export_document: true,
            },
            ToolId::Pen,
            BrushSettings::try_from(sketch_core::Brush {
                rgba: [0., 0., 0., 1.],
                width: 4.,
            })
            .unwrap(),
        )
        .unwrap();
    (runtime, window, root, store, identity)
}
pub fn file(r: &mut NativeRuntime, w: u64, i: FileIntent) -> ActionResult {
    r.dispatch(w, ctx(r, w), ApplicationCommand::File(i))
        .unwrap()
}
pub fn save_as(r: &mut NativeRuntime, w: u64, path: std::path::PathBuf) {
    let IntentEffect::Destination(token, _) = file(r, w, FileIntent::SaveAs).effect else {
        panic!()
    };
    r.supply_location(w, ctx(r, w), token, 111, path).unwrap();
    let event = ready(r);
    r.apply_event(w, event.id).unwrap();
    let event = ready(r);
    r.apply_event(w, event.id).unwrap();
}
