use super::{NativeRuntime, owner_types::*, work::*};
use sketch_core::application::storage::DestinationId;
impl NativeRuntime {
    pub(super) fn new_grant(&mut self, grant: u64) -> Result<(), RuntimeError> {
        if grant == 0 || self.leases.iter().any(|lease| lease.grant == grant) {
            return Err(RuntimeError::Invalid);
        }
        if self.leases.len() >= self.config.sessions * 2 + self.config.operations {
            return Err(RuntimeError::Capacity);
        }
        self.leases.push(Lease {
            grant,
            destination: None,
            references: 1,
        });
        Ok(())
    }
    pub(super) fn retain_grant(&mut self, grant: u64) -> Result<(), RuntimeError> {
        let lease = self
            .leases
            .iter_mut()
            .find(|lease| lease.grant == grant)
            .ok_or(RuntimeError::Invalid)?;
        lease.references = lease
            .references
            .checked_add(1)
            .ok_or(RuntimeError::Capacity)?;
        Ok(())
    }
    pub(super) fn release_grant(&mut self, grant: u64) -> Result<(), RuntimeError> {
        if grant == 0 {
            return Ok(());
        }
        let Some(index) = self.leases.iter().position(|lease| lease.grant == grant) else {
            return Err(RuntimeError::Invalid);
        };
        self.leases[index].references -= 1;
        if self.leases[index].references == 0 {
            let lease = self.leases.remove(index);
            if let Some(destination) = lease.destination {
                self.enqueue(
                    OperationOwner {
                        window: 0,
                        session: 0,
                        request: 0,
                        grant: 0,
                    },
                    OperationKind::Retirement,
                    OperationState::Queued(Box::new(Work::ReleaseDestination(destination))),
                    0,
                )?;
            }
            let id = self.id()?;
            self.events.push_back(NativeEvent {
                id,
                operation: 0,
                window: 0,
                session: 0,
                request: 0,
                grant,
                kind: EventKind::GrantRelease,
                failure: None,
            });
        }
        Ok(())
    }
    pub(super) fn bind_grant(
        &mut self,
        window: u64,
        grant: u64,
        destination: Option<DestinationId>,
    ) -> Result<(), RuntimeError> {
        let index = self.slot(window)?;
        let next = if destination.is_some() { grant } else { 0 };
        let old = self.sessions[index].bound_grant;
        if next != old {
            if next != 0 {
                self.retain_grant(next)?;
            }
            self.sessions[index].bound_grant = next;
            self.release_grant(old)?;
        }
        Ok(())
    }
}
