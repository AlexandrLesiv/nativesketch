use super::inventory::read_bounded;
use super::{RecoveryError, RecoveryId};
use std::{fs, path::Path};
pub(super) struct Resolution {
    pub generation: u64,
    pub seconds: u64,
}
fn checksum(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf29ce484222325, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x100000001b3)
    })
}
pub(super) fn encode(id: RecoveryId, generation: u64, seconds: u64) -> Vec<u8> {
    let mut bytes = b"NSRESOL1".to_vec();
    bytes.extend(id.name().as_bytes());
    bytes.extend(generation.to_le_bytes());
    bytes.extend(seconds.to_le_bytes());
    bytes.extend(checksum(&bytes).to_le_bytes());
    bytes
}
pub(super) fn read(path: &Path, id: RecoveryId) -> Result<Option<Resolution>, RecoveryError> {
    match fs::symlink_metadata(path) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(_) => return Err(RecoveryError::Io),
        Ok(_) => {}
    }
    let bytes = read_bounded(path)?;
    if bytes.len() != 64 || &bytes[..8] != b"NSRESOL1" || bytes[8..40] != *id.name().as_bytes() {
        return Err(RecoveryError::Invalid);
    }
    let number = |at| u64::from_le_bytes(bytes[at..at + 8].try_into().expect("checked marker"));
    if number(56) != checksum(&bytes[..56]) || number(40) == 0 {
        return Err(RecoveryError::Invalid);
    }
    Ok(Some(Resolution {
        generation: number(40),
        seconds: number(48),
    }))
}
