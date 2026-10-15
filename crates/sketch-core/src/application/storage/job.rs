use super::{SaveResult, SaveToken, WriteOutcome};
use crate::application::DocumentSnapshot;

/// Unique owned worker input. No editor reference, filesystem path, or native object.
/// Hosts move this to their serial worker; this type does not spawn a thread.
#[derive(Debug)]
pub struct WriteJob {
    pub(super) token: SaveToken,
    pub(super) snapshot: DocumentSnapshot,
}
impl WriteJob {
    pub fn token(&self) -> SaveToken {
        self.token
    }
    /// Worker-only expensive operation. Returned bytes retain the portable codec format.
    pub fn encode(&self) -> crate::Result<Vec<u8>> {
        self.snapshot.encode()
    }
    /// Drops the owned snapshot before posting the small receipt back to the owner.
    pub fn complete(self, outcome: WriteOutcome) -> WriteReceipt {
        WriteReceipt {
            token: self.token,
            outcome,
        }
    }
}
#[derive(Debug)]
pub struct WriteReceipt {
    pub(super) token: SaveToken,
    pub(super) outcome: WriteOutcome,
}
/// Issued once by the coordinator, consumed once by the editor owner.
#[derive(Debug)]
pub struct SaveCompletion {
    pub(super) token: SaveToken,
    pub(super) result: SaveResult,
}
impl SaveCompletion {
    pub fn token(&self) -> SaveToken {
        self.token
    }
    pub fn result(&self) -> SaveResult {
        self.result
    }
}
