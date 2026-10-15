use super::{
    io_state::IoState,
    work::{NativeFailure, WorkerValue},
};
use crate::storage::{self, DestinationAccess, StoreHooks};
use sketch_core::application::storage::WriteOutcome;
use std::{fs::File, io::Read, path::PathBuf};
impl IoState {
    pub fn export_diagnostics(&self, path: PathBuf, hooks: &dyn StoreHooks) -> WorkerValue {
        let result = (|| {
            let root = self
                .config
                .diagnostics_root
                .as_ref()
                .ok_or(NativeFailure::Unsupported)?;
            let mut bytes = Vec::new();
            for index in (0..4).rev() {
                let path = root.join(format!("diagnostics.{index}.jsonl"));
                DestinationAccess::new(path.clone()).map_err(|_| NativeFailure::Access)?;
                let file = match File::open(path) {
                    Ok(file) => file,
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
                    Err(_) => return Err(NativeFailure::Access),
                };
                if file.metadata().map_err(|_| NativeFailure::Access)?.len() > 256 * 1024 {
                    return Err(NativeFailure::Capacity);
                }
                let before = bytes.len();
                file.take(256 * 1024 + 1)
                    .read_to_end(&mut bytes)
                    .map_err(|_| NativeFailure::Access)?;
                if bytes.len() - before > 256 * 1024 {
                    return Err(NativeFailure::Capacity);
                }
            }
            let access = DestinationAccess::new(path).map_err(|_| NativeFailure::Access)?;
            match storage::write(&access, &bytes, hooks).outcome {
                WriteOutcome::Written(_) => Ok(()),
                _ => Err(NativeFailure::Access),
            }
        })();
        result
            .map(|_| WorkerValue::Complete)
            .unwrap_or_else(WorkerValue::Failure)
    }
}
