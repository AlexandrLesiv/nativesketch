use super::*;

impl SaveCoordinator {
    /// At most one active job. Completed/cancelled entries do not block later writes.
    pub fn start_next(&mut self) -> Option<WriteJob> {
        if self
            .entries
            .iter()
            .any(|entry| matches!(entry.phase, Phase::Active { .. }))
        {
            return None;
        }
        let entry = self
            .entries
            .iter_mut()
            .find(|entry| matches!(entry.phase, Phase::Queued(_)))?;
        let Phase::Queued(snapshot) =
            std::mem::replace(&mut entry.phase, Phase::Active { cancelled: false })
        else {
            unreachable!()
        };
        Some(WriteJob {
            token: entry.token,
            snapshot,
        })
    }
    /// Queued cancellation drops its snapshot. Active cancellation awaits the worker receipt.
    pub fn cancel(&mut self, token: SaveToken) -> Result<(), StorageError> {
        let entry = self.entry_mut(token)?;
        match &mut entry.phase {
            Phase::Queued(_) => {
                entry.phase = Phase::Finished(SaveResult::CancelledBeforeStart);
                let bytes = std::mem::take(&mut entry.bytes);
                self.reserved_bytes -= bytes;
            }
            Phase::Active { cancelled } => *cancelled = true,
            Phase::Finished(_) => return Err(StorageError::InvalidTransition),
        }
        Ok(())
    }
    /// Only a receipt made by consuming the issued worker job can complete an active write.
    pub fn finish(&mut self, receipt: WriteReceipt) -> Result<(), RejectedReceipt> {
        let validation = self.validate_receipt(&receipt);
        if let Err(error) = validation {
            return Err(RejectedReceipt { error, receipt });
        }
        self.apply_receipt(receipt);
        Ok(())
    }
    fn validate_receipt(&mut self, receipt: &WriteReceipt) -> Result<(), StorageError> {
        if receipt.outcome == WriteOutcome::DispatchedUnconfirmed
            && receipt.token.purpose() != SavePurpose::ExportCopy
        {
            return Err(StorageError::CapabilityDenied);
        }
        let entry = self.entry_mut(receipt.token)?;
        let Phase::Active { cancelled } = entry.phase else {
            return Err(StorageError::InvalidTransition);
        };
        let _ = cancelled;
        Ok(())
    }
    fn apply_receipt(&mut self, receipt: WriteReceipt) {
        let entry = self
            .entry_mut(receipt.token)
            .expect("validated receipt token");
        let Phase::Active { cancelled } = entry.phase else {
            unreachable!("validated active receipt")
        };
        let result = match (cancelled, receipt.outcome) {
            (false, WriteOutcome::DispatchedUnconfirmed) => SaveResult::DispatchedUnconfirmed,
            (true, WriteOutcome::DispatchedUnconfirmed) => SaveResult::DispatchedAfterCancellation,
            (false, WriteOutcome::Written(d)) => SaveResult::Written(d),
            (false, WriteOutcome::Uncertain(e)) => SaveResult::Uncertain(e),
            (false, WriteOutcome::Failed(e)) => SaveResult::Failed(e),
            (true, WriteOutcome::Written(d)) => SaveResult::WrittenAfterCancellation(d),
            (true, WriteOutcome::Failed(e)) => SaveResult::FailedAfterCancellation(e),
            (true, WriteOutcome::Uncertain(e)) => SaveResult::UncertainAfterCancellation(e),
        };
        entry.phase = Phase::Finished(result);
        let bytes = std::mem::take(&mut entry.bytes);
        self.reserved_bytes -= bytes;
    }
    /// Frees one slot; the returned completion is not Clone and is consumed by the owner.
    pub fn take_completion(&mut self, token: SaveToken) -> Result<SaveCompletion, StorageError> {
        let index = self
            .entries
            .iter()
            .position(|entry| entry.token == token)
            .ok_or(StorageError::UnknownToken)?;
        let Phase::Finished(result) = self.entries[index].phase else {
            return Err(StorageError::InvalidTransition);
        };
        self.entries.remove(index);
        Ok(SaveCompletion { token, result })
    }
}
