//! Shared bounded ownership for native and Wasm v2 exports. No host policy.
#![forbid(unsafe_code)]
mod buffers;
mod exchange;
mod objects;
mod owner;
mod registry;
mod uploads;
pub use buffers::{buffer_copy, buffer_free, buffer_len, input_free, result_retire_stopped};
pub use exchange::{WorkOutput, app_complete, app_take_work, work_call, work_copy_input};
pub use owner::{app_call, app_free, app_new};
pub use sketch_app_wire::{Failure, WireError};
pub use uploads::{upload_copy, upload_free, upload_len, upload_new, upload_pointer, with_upload};
pub type Handle = u64;
pub type Result<T> = std::result::Result<T, WireError>;
pub(crate) fn error(status: u32) -> WireError {
    WireError {
        status,
        failure: Failure::default(),
    }
}
pub(crate) fn control(bytes: &[u8]) -> Result<()> {
    if bytes.len() > 1024 * 1024 {
        Err(error(3))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests;

/// Preserve correlation only for an exact, bounded v2 frame; payload validation is separate.
pub fn correlation(packet: &[u8]) -> (u32, u64) {
    if packet.len() < 24 || packet.len() > 1024 * 1024 || &packet[..8] != b"NSAP\x02\0\0\0" {
        return (0, 0);
    }
    let length = u32::from_le_bytes(packet[12..16].try_into().expect("header length"));
    if usize::try_from(length).ok().and_then(|n| n.checked_add(24)) != Some(packet.len()) {
        return (0, 0);
    }
    (
        u32::from_le_bytes(packet[8..12].try_into().expect("header opcode")),
        u64::from_le_bytes(packet[16..24].try_into().expect("header request")),
    )
}
