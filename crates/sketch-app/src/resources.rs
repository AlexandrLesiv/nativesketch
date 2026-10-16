use crate::*;
pub(crate) struct Resource {
    pub handle: DestinationHandle,
    pub destination: DestinationId,
    pub descriptor: ResourceDescriptor,
    pub releasing: bool,
}
impl PortableApp {
    pub(crate) fn prepare_resource(
        &mut self,
        descriptor: ResourceDescriptor,
    ) -> Result<Resource, AppError> {
        if descriptor.provider.0 == 0
            || descriptor.resource.0 == 0
            || descriptor.canonical.0 == 0
            || descriptor.generation == 0
            || descriptor.access.confirmed_write && !descriptor.access.write
            || !descriptor.access.read && !descriptor.access.write
        {
            return Err(AppError::Invalid);
        }
        if self.resources.len() >= self.limits.destinations {
            return Err(AppError::Capacity);
        }
        if self.resources.iter().any(|r| {
            r.descriptor.provider == descriptor.provider
                && r.descriptor.resource == descriptor.resource
                && r.descriptor.generation == descriptor.generation
        }) {
            return Err(AppError::Busy);
        }
        let handle =
            DestinationHandle(crate::ids::reserve(&mut self.next_resource)?, self.identity);
        let destination = match self.resources.iter().find(|r| {
            !r.releasing
                && r.descriptor.provider == descriptor.provider
                && r.descriptor.canonical == descriptor.canonical
        }) {
            Some(r) => r.destination,
            None => DestinationId(crate::ids::reserve(&mut self.next_destination)?),
        };
        Ok(Resource {
            handle,
            destination,
            descriptor,
            releasing: false,
        })
    }
    pub(crate) fn resource(&self, handle: DestinationHandle) -> Result<&Resource, AppError> {
        self.resources
            .iter()
            .find(|r| r.handle == handle && !r.releasing)
            .ok_or(AppError::Stale)
    }
    pub(crate) fn destination(&self, id: DestinationId) -> Result<DestinationHandle, AppError> {
        self.resources
            .iter()
            .find(|r| r.destination == id && !r.releasing)
            .map(|r| r.handle)
            .ok_or(AppError::Stale)
    }
    pub(crate) fn write_resource(
        &self,
        id: DestinationId,
        purpose: SavePurpose,
    ) -> Result<DestinationHandle, AppError> {
        self.resources
            .iter()
            .find(|r| {
                r.destination == id
                    && !r.releasing
                    && Self::check_write(r.descriptor.access, purpose).is_ok()
            })
            .map(|r| r.handle)
            .ok_or(AppError::CapabilityDenied)
    }
    pub(crate) fn bound_write_resource(&self, slot: usize) -> Result<DestinationHandle, AppError> {
        let handle = self.sessions[slot]
            .bound_resource
            .ok_or(AppError::CapabilityDenied)?;
        let resource = self.resource(handle)?;
        Self::check_write(resource.descriptor.access, SavePurpose::Save)?;
        Ok(handle)
    }
    pub(crate) fn check_write(
        access: ResourceAccess,
        purpose: SavePurpose,
    ) -> Result<(), AppError> {
        if !access.write || purpose != SavePurpose::ExportCopy && !access.confirmed_write {
            Err(AppError::CapabilityDenied)
        } else {
            Ok(())
        }
    }
    pub(crate) fn release_unused(&mut self) {
        for resource in &mut self.resources {
            let operation_uses = self
                .operations
                .iter()
                .any(|op| op.resource == Some(resource.handle));
            let bound = self
                .sessions
                .iter()
                .any(|slot| slot.bound_resource == Some(resource.handle));
            if !operation_uses && !bound {
                resource.releasing = true;
            }
        }
    }
    /// One reserved slot per accepted lease; no release can be lost to queue overflow.
    /// Host verifies descriptor generation before releasing its retained native resource.
    pub fn take_release(&mut self) -> Option<ResourceRelease> {
        let index = self.resources.iter().position(|r| r.releasing)?;
        let resource = self.resources.swap_remove(index);
        Some(ResourceRelease {
            handle: resource.handle,
            descriptor: resource.descriptor,
        })
    }
}
