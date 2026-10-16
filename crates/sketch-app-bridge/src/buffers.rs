use crate::{objects::*, registry::*, *};
use std::sync::atomic::Ordering;
pub fn buffer_len(id: Handle) -> Result<usize> {
    let held = Checkout::take(id, Kind::Buffer)?;
    let Some(Object::Buffer(reply)) = &held.object else {
        unreachable!()
    };
    Ok(reply.bytes().len())
}
pub fn buffer_copy(id: Handle, offset: usize, destination: &mut [u8]) -> Result<usize> {
    let held = Checkout::take(id, Kind::Buffer)?;
    let Some(Object::Buffer(reply)) = &held.object else {
        unreachable!()
    };
    let tail = reply.bytes().get(offset..).ok_or(error(1))?;
    let count = tail.len().min(destination.len());
    destination[..count].copy_from_slice(&tail[..count]);
    Ok(count)
}
pub fn buffer_free(id: Handle) -> Result<()> {
    free(id, Kind::Buffer)
}
pub fn input_free(id: Handle) -> Result<()> {
    free(id, Kind::Input)
}
fn free(id: Handle, kind: Kind) -> Result<()> {
    Checkout::take(id, kind)?.consume();
    Ok(())
}
pub fn result_retire_stopped(id: Handle) -> Result<()> {
    let mut held = Checkout::take(id, Kind::Result)?;
    if !matches!(held.object, Some(Object::Result(Some(_)))) {
        return Err(error(2));
    }
    if held.owner().life.load(Ordering::Acquire) != STOPPED {
        return Err(error(9));
    }
    held.consume();
    Ok(())
}
