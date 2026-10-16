use super::*;
use crate::application::storage::{
    Acknowledgement, DestinationId, SaveCompletion, SavePurpose, SaveResult, SaveState,
    StorageError,
};
impl EditorSession {
    /// A user-selected validated recovery becomes an editable, dirty, unbound new session.
    pub fn new_recovered(
        capabilities: Capabilities,
        tool: ToolId,
        brush: BrushSettings,
        document: ValidatedDocument,
    ) -> Result<Self, EditorError> {
        if !capabilities.edit_document {
            return Err(EditorError {
                operation: Operation::Construct,
                kind: FailureKind::ReadOnly,
                context: None,
            });
        }
        let mut session = Self::new(capabilities, tool, brush, Some(document))?;
        session.editor.history.acknowledge_snapshot(None);
        Ok(session)
    }
    pub fn save_state(&self) -> SaveState {
        self.save
    }
    /// Trusted host calls after a validated load, before edits. No path enters Rust state.
    pub fn bind_loaded_destination(
        &mut self,
        destination: DestinationId,
    ) -> Result<(), StorageError> {
        if self.save.destination.is_some()
            || self.save.pending_binding.is_some()
            || self.editor.summary().dirty
            || self.require_idle().is_err()
        {
            return Err(StorageError::InvalidBinding);
        }
        self.save.binding = self
            .save
            .binding
            .checked_add(1)
            .ok_or(StorageError::ResourceLimit)?;
        self.save.destination = Some(destination);
        Ok(())
    }
    pub(crate) fn snapshot_memory(&self) -> Result<usize, StorageError> {
        crate::persistence::snapshot_bytes(self.editor.document())
            .ok_or(StorageError::ResourceLimit)
    }
    pub(crate) fn storage_snapshot(&self) -> DocumentSnapshot {
        DocumentSnapshot {
            document: self.editor.document().clone(),
            session: self.session,
            revision: Revision(self.editor.summary().revision),
        }
    }
    pub(crate) fn reserve_binding(&mut self, sequence: u64, destination: DestinationId) {
        self.save.pending_binding = Some(sequence);
        self.save.pending_destination = Some(destination);
    }
    /// Consumes an issued completion on the mutation owner; no I/O or document traversal.
    /// May run during a new draft: committed saved-state changes do not touch the draft.
    pub fn acknowledge_save(&mut self, completion: SaveCompletion) -> Acknowledgement {
        let token = completion.token();
        if token.session() != self.session {
            return Acknowledgement::StaleSession;
        }
        if self.save.pending_binding == Some(token.sequence()) {
            self.save.pending_binding = None;
            self.save.pending_destination = None;
        }
        if token.binding_generation() != self.save.binding {
            if token.sequence() > self.save.last_ack
                && matches!(
                    completion.result(),
                    SaveResult::Written(_)
                        | SaveResult::WrittenAfterCancellation(_)
                        | SaveResult::Uncertain(_)
                        | SaveResult::UncertainAfterCancellation(_)
                )
            {
                // A late write may target the same canonical destination after Save As.
                // It cannot authorize clean state, but it invalidates old clean knowledge.
                self.invalidate_written_copy(token.destination(), token.sequence());
            }
            return Acknowledgement::StaleBinding;
        }
        match completion.result() {
            SaveResult::DispatchedUnconfirmed => return Acknowledgement::CopyDispatched,
            SaveResult::DispatchedAfterCancellation => return Acknowledgement::Cancelled,
            SaveResult::Written(_) => {}
            SaveResult::WrittenAfterCancellation(_) => {
                self.invalidate_written_copy(token.destination(), token.sequence());
                return Acknowledgement::Cancelled;
            }
            SaveResult::Uncertain(_) | SaveResult::UncertainAfterCancellation(_) => {
                self.invalidate_written_copy(token.destination(), token.sequence());
                return Acknowledgement::Failed;
            }
            SaveResult::CancelledBeforeStart | SaveResult::FailedAfterCancellation(_) => {
                return Acknowledgement::Cancelled;
            }
            SaveResult::Failed(_) => return Acknowledgement::Failed,
        }
        match token.purpose() {
            SavePurpose::Checkpoint => Acknowledgement::CheckpointWritten,
            SavePurpose::ExportCopy => {
                self.invalidate_written_copy(token.destination(), token.sequence());
                Acknowledgement::CopyWritten
            }
            SavePurpose::Save | SavePurpose::SaveAs => {
                let changed_destination = self.save.destination != Some(token.destination());
                if token.purpose() == SavePurpose::SaveAs {
                    // Capacity was checked before issuing the destination-changing token.
                    self.save.binding += 1;
                    self.save.destination = Some(token.destination());
                } else if changed_destination {
                    return Acknowledgement::StaleBinding;
                } else if token.sequence() <= self.save.last_ack {
                    return Acknowledgement::Superseded;
                }
                // A later write to the OLD destination cannot supersede a Save As binding.
                // A later write to the SAME destination still owns its saved revision.
                if changed_destination || token.sequence() > self.save.last_ack {
                    self.save.last_ack = token.sequence();
                    self.editor
                        .history
                        .acknowledge_snapshot(Some(token.revision().0));
                }
                if self.editor.summary().dirty {
                    Acknowledgement::SavedOlder
                } else {
                    Acknowledgement::SavedCurrent
                }
            }
        }
    }
    fn invalidate_written_copy(&mut self, destination: DestinationId, sequence: u64) {
        if self.save.destination == Some(destination) && sequence > self.save.last_ack {
            self.save.last_ack = sequence;
            self.editor.history.acknowledge_snapshot(None);
        }
    }
}
