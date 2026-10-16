use crate::*;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WireError {
    pub status: u32,
    pub failure: Failure,
}
impl WireError {
    pub(crate) fn plain(status: u32) -> Self {
        Self {
            status,
            failure: Failure::default(),
        }
    }
    pub(crate) fn capacity() -> Self {
        Self::plain(3)
    }
    pub(crate) fn malformed() -> Self {
        Self::transport(1)
    }
    pub(crate) fn transport(code: u32) -> Self {
        Self {
            status: 1,
            failure: Failure::new(6, code),
        }
    }
    /// Fixed stack response even when ordinary reply capacity has been exhausted.
    pub fn packet(self, opcode: u32, request: u64) -> [u8; 72] {
        let mut bytes = [0; 72];
        bytes[..4].copy_from_slice(b"NSAP");
        bytes[4] = 2;
        bytes[8..12].copy_from_slice(&(opcode | 0x80000000).to_le_bytes());
        bytes[12..16].copy_from_slice(&48u32.to_le_bytes());
        bytes[16..24].copy_from_slice(&request.to_le_bytes());
        bytes[24..28].copy_from_slice(&self.status.to_le_bytes());
        bytes[32..].copy_from_slice(&self.failure.bytes());
        bytes
    }
}
impl From<AppError> for WireError {
    fn from(value: AppError) -> Self {
        let failure = match value {
            AppError::Core(v) => Failure::from(v.failure).context(v.context),
            AppError::Storage(v) => Failure::new(2, crate::tags::storage_error(v)),
            AppError::Editor(v) => v.into(),
            AppError::Scene(v) => Failure::new(
                5,
                match v {
                    sketch_core::SceneError::InvalidViewport => 1,
                    sketch_core::SceneError::ResourceLimit => 2,
                },
            ),
            _ => Failure::default(),
        };
        let status = match value {
            AppError::Invalid => 1,
            AppError::Stale => 7,
            AppError::Busy => 9,
            AppError::Capacity => 3,
            AppError::CapabilityDenied => 8,
            AppError::Unsupported => 13,
            AppError::Core(_) | AppError::Storage(_) | AppError::Editor(_) | AppError::Scene(_) => {
                status(failure)
            }
        };
        Self { status, failure }
    }
}
fn status(f: Failure) -> u32 {
    match (f.domain, f.code) {
        (1, 1) | (2, 3) | (4, 1) | (5, 1) => 1,
        (1, 2) | (2, 7) | (4, 5) => 2,
        (1, 3) | (2, 2) | (4, 4) => 8,
        (1, 4 | 5) | (2, 6) | (4, 2) => 7,
        (1, 6) | (2, 5) | (3, 3) | (4, 6) | (5, 2) => 3,
        (1, 7) | (3, 1) => 5,
        (1, 8) | (3, 2 | 4 | 5 | 6) => 4,
        (2, 1 | 4) | (4, 3) => 9,
        _ => 1,
    }
}
