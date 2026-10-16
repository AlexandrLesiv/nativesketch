use super::{io_state::IoState, work::NativeFailure};
use crate::recovery::CheckpointMetadata;
use sketch_core::application::{
    ValidatedDocument,
    routing::{PreparedReplacement, ReplacementFailure, ReplacementSource, ReplacementTicket},
    storage::StorageFailure,
};
use std::{
    fs::File,
    io::Read,
    panic::{AssertUnwindSafe, catch_unwind},
};
impl IoState {
    pub fn prepare(
        &self,
        ticket: ReplacementTicket,
        recovery: Option<CheckpointMetadata>,
        after_validation: impl FnOnce() -> Result<(), NativeFailure>,
    ) -> Result<PreparedReplacement, ReplacementFailure> {
        let document = catch_unwind(AssertUnwindSafe(|| match ticket.source() {
            ReplacementSource::New => Ok(None),
            // Portable providers own viewer sources; local native v1 has no such route.
            ReplacementSource::ViewSource(_) => Err(NativeFailure::Unsupported),
            ReplacementSource::Document(id) => {
                let entry = self
                    .destinations
                    .iter()
                    .find(|entry| entry.id == id)
                    .ok_or(NativeFailure::Access)?;
                entry.access.validate().map_err(|_| NativeFailure::Access)?;
                let file = File::open(&entry.path).map_err(|_| NativeFailure::Access)?;
                if file.metadata().map_err(|_| NativeFailure::Access)?.len() > 64 * 1024 * 1024 {
                    return Err(NativeFailure::Capacity);
                }
                let mut bytes = Vec::new();
                file.take(64 * 1024 * 1024 + 1)
                    .read_to_end(&mut bytes)
                    .map_err(|_| NativeFailure::Access)?;
                if bytes.len() > 64 * 1024 * 1024 {
                    return Err(NativeFailure::Capacity);
                }
                ValidatedDocument::decode(&bytes)
                    .map(Some)
                    .map_err(|_| NativeFailure::Corrupt)
            }
            ReplacementSource::Recovery(_) => {
                let metadata = recovery.ok_or(NativeFailure::Stale)?;
                let store = &self.recovery.as_ref().map_err(|error| *error)?.store;
                let candidate = store
                    .load(metadata.identity)
                    .map_err(NativeFailure::Recovery)?;
                let found = candidate.envelope.metadata();
                if found.generation != metadata.generation || found.revision != metadata.revision {
                    return Err(NativeFailure::Stale);
                }
                candidate
                    .envelope
                    .into_document()
                    .map(Some)
                    .map_err(NativeFailure::Recovery)
            }
        }))
        .unwrap_or(Err(NativeFailure::Panic));
        match document {
            Ok(document) => {
                if document.is_some() && after_validation().is_err() {
                    return Err(ticket.fail(StorageFailure::Access));
                }
                ticket.prepare(document)
            }
            Err(error) => Err(ticket.fail(match error {
                NativeFailure::Capacity => StorageFailure::Capacity,
                NativeFailure::Corrupt => StorageFailure::Encode,
                NativeFailure::Panic => StorageFailure::WorkerStopped,
                _ => StorageFailure::Access,
            })),
        }
    }
}
