use crate::{boundary::*, convert, dto};
use sketch_services::native_runtime::{NativeRuntime, RuntimeConfig};
/// # Safety
/// Valid aligned buffers, unique handles and serialized owner-thread use are required.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sketch_runtime_v1_create(
    config: *const dto::RuntimeConfig,
    output: *mut *mut Runtime,
) -> u32 {
    unsafe {
        entry(|| {
            require_output(output)?;
            output.write(std::ptr::null_mut());
            let value = read(config)?;
            if value.struct_size as usize != std::mem::size_of::<dto::RuntimeConfig>() {
                return Err(1);
            }
            let defaults = RuntimeConfig::default();
            let recovery = text(value.recovery_root, 16384)?;
            let diagnostics = text(value.diagnostics_root, 16384)?;
            let configured = RuntimeConfig {
                sessions: if value.max_sessions == 0 {
                    defaults.sessions
                } else {
                    value.max_sessions as usize
                },
                operations: if value.max_operations == 0 {
                    defaults.operations
                } else {
                    value.max_operations as usize
                },
                events: if value.max_events == 0 {
                    defaults.events
                } else {
                    value.max_events as usize
                },
                snapshot_bytes: if value.snapshot_bytes == 0 {
                    defaults.snapshot_bytes
                } else {
                    usize::try_from(value.snapshot_bytes).map_err(|_| 1u32)?
                },
                build: value.build_id,
                recovery_root: if recovery.is_empty() {
                    None
                } else {
                    Some(recovery.into())
                },
                diagnostics_root: if diagnostics.is_empty() {
                    None
                } else {
                    Some(diagnostics.into())
                },
            };
            let core = status(NativeRuntime::new(configured))?;
            output.write(Box::into_raw(Box::new(Runtime {
                core,
                owner: std::thread::current().id(),
                poisoned: false,
            })));
            Ok(())
        })
    }
}
/// # Safety
/// Valid aligned buffers, unique handles and serialized owner-thread use are required.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sketch_runtime_v1_state(
    pointer: *mut Runtime,
    output: *mut dto::RuntimeState,
) -> u32 {
    unsafe {
        runtime(pointer, true, |value| {
            require_output(output)?;
            output.write(convert::runtime::state(status(value.core.state())?));
            Ok(())
        })
    }
}
/// # Safety
/// Valid aligned buffers, unique handles and serialized owner-thread use are required.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sketch_runtime_v1_poll(
    pointer: *mut Runtime,
    output: *mut dto::Event,
    capacity: u32,
    count: *mut u32,
    state: *mut dto::RuntimeState,
) -> u32 {
    unsafe {
        runtime(pointer, true, |value| {
            require_output(count)?;
            require_output(state)?;
            if capacity > 0 {
                require_output(output)?;
            }
            let events = status(value.core.poll(capacity as usize))?;
            for (index, event) in events.iter().enumerate() {
                output.add(index).write(convert::runtime::event(*event));
            }
            count.write(events.len() as u32);
            state.write(convert::runtime::state(status(value.core.state())?));
            Ok(())
        })
    }
}
/// # Safety
/// Valid aligned buffers, unique handles and serialized owner-thread use are required.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sketch_runtime_v1_idle(
    pointer: *mut Runtime,
    output: *mut dto::RuntimeState,
) -> u32 {
    unsafe {
        runtime(pointer, false, |value| {
            require_output(output)?;
            status(value.core.idle())?;
            output.write(convert::runtime::state(status(value.core.state())?));
            Ok(())
        })
    }
}
/// # Safety
/// Valid aligned buffers, unique handles and serialized owner-thread use are required.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sketch_runtime_v1_release_event(pointer: *mut Runtime, event: u64) -> u32 {
    unsafe {
        runtime(pointer, true, |value| {
            status(value.core.release_event(event))
        })
    }
}
/// # Safety
/// Valid aligned buffers, unique handles and serialized owner-thread use are required.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sketch_runtime_v1_request_shutdown(pointer: *mut Runtime) -> u32 {
    unsafe { runtime(pointer, true, |value| status(value.core.request_shutdown())) }
}
/// # Safety
/// Valid aligned buffers, unique handles and serialized owner-thread use are required.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sketch_runtime_v1_free(pointer: *mut *mut Runtime) -> u32 {
    unsafe {
        entry(|| {
            let handle = read(pointer)?;
            if handle.is_null() {
                return Ok(());
            }
            if (*handle).owner != std::thread::current().id() {
                return Err(11);
            }
            if !status((*handle).core.free_ready())? {
                return Err(9);
            }
            drop(Box::from_raw(handle));
            pointer.write(std::ptr::null_mut());
            Ok(())
        })
    }
}
