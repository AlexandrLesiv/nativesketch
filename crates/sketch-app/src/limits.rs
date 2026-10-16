use crate::*;
#[derive(Debug, Clone, Copy)]
pub struct AppLimits {
    pub sessions: usize,
    pub operations: usize,
    pub destinations: usize,
    pub buffers: usize,
    pub buffer_bytes: usize,
    pub document_bytes: usize,
    pub storage: StorageLimits,
}
impl Default for AppLimits {
    fn default() -> Self {
        Self {
            sessions: 16,
            operations: 32,
            destinations: 64,
            buffers: 32,
            buffer_bytes: 128 * 1024 * 1024,
            document_bytes: 64 * 1024 * 1024,
            storage: StorageLimits {
                jobs: 16,
                snapshot_bytes: 128 * 1024 * 1024,
            },
        }
    }
}
impl AppLimits {
    pub(crate) fn validate(self) -> Result<Self, AppError> {
        if self.sessions == 0
            || self.sessions > 1024
            || self.operations == 0
            || self.operations > 1024
            || self.destinations == 0
            || self.destinations > 2048
            || self.buffers == 0
            || self.buffers > 1024
            || self.document_bytes < 24
            || self.document_bytes > 64 * 1024 * 1024
            || self.buffer_bytes < self.document_bytes
        {
            Err(AppError::Capacity)
        } else {
            Ok(self)
        }
    }
}
