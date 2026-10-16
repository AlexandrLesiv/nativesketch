use crate::{objects::*, registry::*, *};
use std::sync::{Mutex, MutexGuard};
static USAGE: Mutex<(usize, usize)> = Mutex::new((0, 0));
fn usage() -> MutexGuard<'static, (usize, usize)> {
    USAGE.lock().unwrap_or_else(|p| p.into_inner())
}
pub fn upload_new(length: usize) -> Result<Handle> {
    let slot = Slot::reserve(Kind::Upload, None)?;
    let mut usage = usage();
    if length > 64 * 1024 * 1024 + 1 || usage.0 == 64 || length > 128 * 1024 * 1024 - usage.1 {
        return Err(error(3));
    }
    let mut bytes = Vec::new();
    bytes.try_reserve_exact(length).map_err(|_| error(3))?;
    bytes.resize(length, 0);
    usage.0 += 1;
    usage.1 += length;
    Ok(slot.commit(Object::Upload(bytes), None))
}
pub fn upload_free(id: Handle) -> Result<()> {
    let mut held = Checkout::take(id, Kind::Upload)?;
    let Some(Object::Upload(bytes)) = &held.object else {
        unreachable!()
    };
    let mut usage = usage();
    usage.0 -= 1;
    usage.1 -= bytes.len();
    held.consume();
    Ok(())
}
pub fn with_upload<T>(id: Handle, f: impl FnOnce(&[u8]) -> Result<T>) -> Result<T> {
    let held = Checkout::take(id, Kind::Upload)?;
    let Some(Object::Upload(bytes)) = &held.object else {
        unreachable!()
    };
    f(bytes)
}
pub fn upload_len(id: Handle) -> Result<usize> {
    with_upload(id, |b| Ok(b.len()))
}
/// Borrowed fixed-size allocation; only Wasm's host writes this memory between calls.
pub fn upload_pointer(id: Handle) -> Result<*const u8> {
    with_upload(id, |b| Ok(b.as_ptr()))
}
pub fn upload_copy(
    buffer: Handle,
    offset: usize,
    upload: Handle,
    capacity: usize,
) -> Result<usize> {
    let mut held = Checkout::take(upload, Kind::Upload)?;
    let Some(Object::Upload(bytes)) = &mut held.object else {
        unreachable!()
    };
    let destination = bytes.get_mut(..capacity).ok_or(error(1))?;
    buffer_copy(buffer, offset, destination)
}
