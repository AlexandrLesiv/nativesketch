use super::*;
use crate::application::{EditorSession, RequestId};
use std::sync::atomic::AtomicU64;
static NEXT_WRITE: AtomicU64 = AtomicU64::new(1);

impl SaveCoordinator {
    /// Owner-only file-action/idle callback. Reserves capacity before O(document) copying.
    /// Never invoke from a pen callback. The snapshot is exact at this synchronous call.
    pub fn request(
        &mut self,
        session: &mut EditorSession,
        request: RequestId,
        purpose: SavePurpose,
        destination: DestinationId,
    ) -> Result<SaveToken, StorageError> {
        let state = session.view_state();
        let binding = session.save_state();
        if state.interaction != crate::application::InteractionState::Idle {
            return Err(StorageError::InvalidTransition);
        }
        match purpose {
            SavePurpose::Checkpoint if !state.capabilities.edit_document => {
                return Err(StorageError::CapabilityDenied);
            }
            SavePurpose::ExportCopy if !state.capabilities.export_document => {
                return Err(StorageError::CapabilityDenied);
            }
            SavePurpose::Save | SavePurpose::SaveAs
                if !state.capabilities.edit_document || !state.capabilities.export_document =>
            {
                return Err(StorageError::CapabilityDenied);
            }
            _ => {}
        }
        if purpose == SavePurpose::Save && binding.destination() != Some(destination) {
            return Err(StorageError::InvalidBinding);
        }
        if purpose == SavePurpose::Checkpoint && binding.destination() == Some(destination) {
            return Err(StorageError::InvalidBinding);
        }
        if matches!(purpose, SavePurpose::ExportCopy | SavePurpose::Checkpoint)
            && binding.pending_destination == Some(destination)
        {
            return Err(StorageError::BindingPending);
        }
        if purpose == SavePurpose::SaveAs {
            if binding.binding_pending() {
                return Err(StorageError::BindingPending);
            }
            binding
                .binding
                .checked_add(1)
                .ok_or(StorageError::ResourceLimit)?;
        }
        if self.entries.len() == self.limits.jobs {
            return Err(StorageError::Busy);
        }
        let bytes = session.snapshot_memory()?;
        let reserved = self
            .reserved_bytes
            .checked_add(bytes)
            .ok_or(StorageError::ResourceLimit)?;
        if reserved > self.limits.snapshot_bytes {
            return Err(StorageError::Busy);
        }
        let sequence =
            super::super::ids::reserve(&NEXT_WRITE).map_err(|_| StorageError::ResourceLimit)?;
        let token = SaveToken {
            session: state.session,
            request,
            revision: state.revision,
            destination,
            binding: binding.binding,
            sequence,
            purpose,
        };
        self.reserved_bytes = reserved;
        let snapshot = session.storage_snapshot();
        self.entries.push_back(Entry {
            token,
            bytes,
            phase: Phase::Queued(snapshot),
        });
        if purpose == SavePurpose::SaveAs {
            session.reserve_binding(sequence, destination);
        }
        Ok(token)
    }
}
