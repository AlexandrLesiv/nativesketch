use crate::{boundary::*, dto};
use sketch_services::recovery::RecoveryId;
/// # Safety
/// Valid aligned buffers, unique handles and serialized owner-thread use are required.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sketch_session_v1_recovery_identity(
    pointer: *mut Runtime,
    handle: *mut Session,
    expected_session: u64,
    identity: *const u8,
    name: dto::Text,
) -> u32 {
    unsafe {
        runtime(pointer, false, |value| {
            let session = session(handle, pointer)?;
            let identity =
                RecoveryId::from_bytes(slice(identity, 16, 16)?.try_into().map_err(|_| 1u32)?)
                    .map_err(|_| 1u32)?;
            status(value.core.register_recovery(
                session.window,
                expected_session,
                identity,
                text(name, 256)?,
            ))
        })
    }
}
/// # Safety
/// Valid aligned buffers, unique handles and serialized owner-thread use are required.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sketch_runtime_v1_maintenance(
    pointer: *mut Runtime,
    request: u64,
    command: *const dto::Maintenance,
    output: *mut u64,
) -> u32 {
    unsafe {
        runtime(pointer, false, |value| {
            require_output(output)?;
            let command = read(command)?;
            if command.reserved != 0 {
                return Err(1);
            }
            let operation = match command.tag {
                1 => {
                    if command.item_id != 0 || command.window != 0 {
                        return Err(1);
                    }
                    status(value.core.list_recoveries(request))?
                }
                2 => status(value.core.discard_recovery(request, command.item_id))?,
                3 => {
                    if command.item_id != 0 {
                        return Err(1);
                    }
                    status(value.core.retry_recovery(request, command.window))?
                }
                4 => {
                    if command.location.provider != 0 {
                        return Err(13);
                    }
                    if command.location.reserved != 0 {
                        return Err(1);
                    }
                    status(value.core.export_diagnostics(
                        request,
                        command.location.grant_id,
                        text(command.location.path, 16384)?.into(),
                    ))?
                }
                _ => return Err(1),
            };
            output.write(operation);
            Ok(())
        })
    }
}
/// # Safety
/// Valid aligned buffers, unique handles and serialized owner-thread use are required.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sketch_runtime_v1_recovery_copy(
    pointer: *mut Runtime,
    event: u64,
    output: *mut dto::RecoveryItem,
    capacity: u32,
    count: *mut u32,
    info: *mut dto::RecoveryInfo,
) -> u32 {
    unsafe {
        runtime(pointer, false, |value| {
            require_output(count)?;
            require_output(info)?;
            let inventory = status(value.core.inventory(event))?;
            count.write(inventory.items.len() as u32);
            if (capacity as usize) < inventory.items.len() {
                return Err(3);
            }
            if !inventory.items.is_empty() {
                require_output(output)?;
            }
            for (index, (id, metadata)) in inventory.items.iter().enumerate() {
                let mut name = [0; 256];
                name[..metadata.display_name.len()]
                    .copy_from_slice(metadata.display_name.as_bytes());
                output.add(index).write(dto::RecoveryItem {
                    item_id: *id,
                    generation: metadata.generation,
                    revision: metadata.revision,
                    created_seconds: metadata.created_seconds,
                    recovery_id: metadata.identity.bytes(),
                    display_name_length: metadata.display_name.len() as u32,
                    rejected_generations: inventory.listing.rejected_generations as u32,
                    display_name: name,
                });
            }
            info.write(dto::RecoveryInfo {
                count: inventory.items.len() as u32,
                rejected_sessions: inventory.listing.rejected_sessions as u32,
                invalid_markers: inventory.listing.invalid_resolution_markers as u32,
                capacity_blocked: u32::from(inventory.listing.capacity_blocked),
                usage_known: u32::from(inventory.listing.usage.is_some()),
                used_bytes: inventory.listing.usage.map_or(0, |usage| usage.bytes),
                limit_bytes: inventory.listing.limit_bytes,
                reserved: 0,
            });
            Ok(())
        })
    }
}
