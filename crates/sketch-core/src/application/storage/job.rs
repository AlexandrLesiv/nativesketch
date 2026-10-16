use super::{SavePurpose, SaveResult, SaveToken, StorageError, WriteOutcome};
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
    /// Checked codec size using stroke headers/sample counts, without encoding/copying.
    pub fn encoded_len(&self) -> crate::Result<usize> {
        crate::persistence::encoded_bytes(&self.snapshot.document)
            .ok_or(crate::Error::InvalidArgument)
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
impl WriteReceipt {
    pub fn token(&self) -> SaveToken {
        self.token
    }
    /// Correct a rejected provider report using the actual observed I/O outcome.
    /// The issued token is immutable; this cannot authorize a different job.
    pub fn revise_outcome(&mut self, outcome: WriteOutcome) -> Result<(), StorageError> {
        if outcome == WriteOutcome::DispatchedUnconfirmed
            && self.token.purpose() != SavePurpose::ExportCopy
        {
            return Err(StorageError::CapabilityDenied);
        }
        self.outcome = outcome;
        Ok(())
    }
}
/// Rejection retains the only issued receipt; ordering stays active until a real
/// corrected completion is supplied. No synthetic success or cancellation is implied.
#[derive(Debug)]
pub struct RejectedReceipt {
    pub(super) error: StorageError,
    pub(super) receipt: WriteReceipt,
}
impl RejectedReceipt {
    pub fn error(&self) -> StorageError {
        self.error
    }
    pub fn into_receipt(self) -> WriteReceipt {
        self.receipt
    }
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
