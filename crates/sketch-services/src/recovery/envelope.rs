use super::{MAX_ENVELOPE, RecoveryError};
use sketch_core::application::ValidatedDocument;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecoveryId([u8; 16]);
impl RecoveryId {
    /// Host must supply cryptographic random bytes; nonzero validation is not entropy proof.
    pub fn from_bytes(bytes: [u8; 16]) -> Result<Self, RecoveryError> {
        if bytes == [0; 16] {
            Err(RecoveryError::Invalid)
        } else {
            Ok(Self(bytes))
        }
    }
    pub fn name(self) -> String {
        self.0.iter().map(|byte| format!("{byte:02x}")).collect()
    }
    pub(crate) fn parse(name: &str) -> Result<Self, RecoveryError> {
        if name.len() != 32
            || !name
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        {
            return Err(RecoveryError::UnsafeEntry);
        }
        let mut bytes = [0; 16];
        for (index, slot) in bytes.iter_mut().enumerate() {
            *slot = u8::from_str_radix(&name[index * 2..index * 2 + 2], 16)
                .map_err(|_| RecoveryError::Invalid)?;
        }
        Self::from_bytes(bytes)
    }
}
/// Display text is local recovery metadata, never a diagnostic field or path component.
#[derive(Clone)]
pub struct CheckpointMetadata {
    pub identity: RecoveryId,
    pub generation: u64,
    pub revision: u64,
    pub created_seconds: u64,
    pub display_name: String,
}
pub struct RecoveryEnvelope {
    pub(crate) metadata: CheckpointMetadata,
    payload: Vec<u8>,
}
fn checksum(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf29ce484222325, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x100000001b3)
    })
}
impl RecoveryEnvelope {
    pub fn metadata(&self) -> &CheckpointMetadata {
        &self.metadata
    }
    pub fn new(metadata: CheckpointMetadata, payload: Vec<u8>) -> Result<Self, RecoveryError> {
        if metadata.generation == 0
            || metadata.display_name.len() > 256
            || metadata.display_name.chars().any(char::is_control)
        {
            return Err(RecoveryError::Invalid);
        }
        ValidatedDocument::decode(&payload).map_err(|_| RecoveryError::Invalid)?;
        Ok(Self { metadata, payload })
    }
    pub fn into_document(self) -> Result<ValidatedDocument, RecoveryError> {
        ValidatedDocument::decode(&self.payload).map_err(|_| RecoveryError::Invalid)
    }
    pub fn encode(&self) -> Vec<u8> {
        let mut bytes = b"NSRECOV1".to_vec();
        bytes.extend(self.metadata.identity.0);
        for value in [
            self.metadata.generation,
            self.metadata.revision,
            self.metadata.created_seconds,
            self.metadata.display_name.len() as u64,
            self.payload.len() as u64,
        ] {
            bytes.extend(value.to_le_bytes());
        }
        bytes.extend(self.metadata.display_name.as_bytes());
        bytes.extend(&self.payload);
        bytes.extend(checksum(&bytes).to_le_bytes());
        bytes
    }
    pub fn decode(bytes: &[u8]) -> Result<Self, RecoveryError> {
        if bytes.len() < 72 || bytes.len() as u64 > MAX_ENVELOPE || &bytes[..8] != b"NSRECOV1" {
            return Err(RecoveryError::Invalid);
        }
        let number =
            |at: usize| u64::from_le_bytes(bytes[at..at + 8].try_into().expect("checked header"));
        if checksum(&bytes[..bytes.len() - 8]) != number(bytes.len() - 8) {
            return Err(RecoveryError::Invalid);
        }
        let name_len = usize::try_from(number(48)).map_err(|_| RecoveryError::Invalid)?;
        let payload_len = usize::try_from(number(56)).map_err(|_| RecoveryError::Invalid)?;
        if name_len > 256
            || payload_len > 64 * 1024 * 1024
            || 72usize
                .checked_add(name_len)
                .and_then(|size| size.checked_add(payload_len))
                != Some(bytes.len())
        {
            return Err(RecoveryError::Invalid);
        }
        let name_end = 64 + name_len;
        let metadata = CheckpointMetadata {
            identity: RecoveryId::from_bytes(
                bytes[8..24]
                    .try_into()
                    .map_err(|_| RecoveryError::Invalid)?,
            )?,
            generation: number(24),
            revision: number(32),
            created_seconds: number(40),
            display_name: std::str::from_utf8(&bytes[64..name_end])
                .map_err(|_| RecoveryError::Invalid)?
                .into(),
        };
        Self::new(metadata, bytes[name_end..bytes.len() - 8].to_vec())
    }
}
