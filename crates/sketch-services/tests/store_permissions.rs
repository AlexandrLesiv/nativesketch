#![cfg(unix)]
mod common;
use common::*;
use sketch_services::storage::{self, DestinationAccess, Stage, StoreHooks};
use std::{fs, io, os::unix::fs::PermissionsExt, path::PathBuf};
struct InspectStage(PathBuf);
impl StoreHooks for InspectStage {
    fn at(&self, stage: Stage) -> io::Result<()> {
        if stage == Stage::AfterCreate {
            let temporary = fs::read_dir(&self.0)?
                .map(Result::unwrap)
                .find(|entry| {
                    entry
                        .file_name()
                        .to_string_lossy()
                        .starts_with(".native-sketch-")
                })
                .unwrap();
            assert_eq!(temporary.metadata()?.permissions().mode() & 0o777, 0o600);
        }
        Ok(())
    }
}
#[test]
fn stage_is_private_and_replacement_preserves_existing_unix_mode() {
    let temp = Temp::new();
    let path = temp.0.join("document");
    for mode in [0o600, 0o640] {
        fs::write(&path, b"old").unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(mode)).unwrap();
        let access = DestinationAccess::new(path.clone()).unwrap();
        let report = storage::write(&access, FIXTURE, &InspectStage(temp.0.clone()));
        assert!(matches!(
            report.outcome,
            sketch_core::application::storage::WriteOutcome::Written(_)
        ));
        assert_eq!(
            fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            mode
        );
    }
}
