mod admission;
mod job;
mod lifecycle;
mod types;
use super::DocumentSnapshot;
pub use job::{SaveCompletion, WriteJob, WriteReceipt};
use std::collections::VecDeque;
pub use types::*;

#[derive(Debug)]
enum Phase {
    Queued(DocumentSnapshot),
    Active { cancelled: bool },
    Finished(SaveResult),
}
#[derive(Debug)]
struct Entry {
    token: SaveToken,
    bytes: usize,
    phase: Phase,
}
/// One process-wide owner serializes admission and completion from all windows.
/// Bounded outstanding slots include completed jobs until their result is consumed.
/// No thread, I/O or encoding is created by this portable coordinator.
#[derive(Debug)]
pub struct SaveCoordinator {
    limits: StorageLimits,
    entries: VecDeque<Entry>,
    reserved_bytes: usize,
}
impl SaveCoordinator {
    pub fn new(limits: StorageLimits) -> Result<Self, StorageError> {
        if limits.jobs == 0 || limits.jobs > 1024 || limits.snapshot_bytes == 0 {
            return Err(StorageError::ResourceLimit);
        }
        Ok(Self {
            limits,
            entries: VecDeque::new(),
            reserved_bytes: 0,
        })
    }
    pub fn outstanding(&self) -> usize {
        self.entries.len()
    }
    /// Includes finished results until consumed; navigation must await their acknowledgements.
    pub fn explicit_outstanding(&self, session: super::DocumentSessionId) -> usize {
        self.entries
            .iter()
            .filter(|entry| {
                entry.token.session() == session && entry.token.purpose() != SavePurpose::Checkpoint
            })
            .count()
    }
    pub fn reserved_snapshot_bytes(&self) -> usize {
        self.reserved_bytes
    }
    fn entry_mut(&mut self, token: SaveToken) -> Result<&mut Entry, StorageError> {
        self.entries
            .iter_mut()
            .find(|entry| entry.token == token)
            .ok_or(StorageError::UnknownToken)
    }
}
#[cfg(test)]
mod tests;
