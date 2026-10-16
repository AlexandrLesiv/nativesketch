use crate::WireError;
#[derive(Debug, Clone, Copy)]
pub struct TransportLimits {
    pub buffers: usize,
    pub buffer_bytes: usize,
    pub scenes: usize,
    pub scene_bytes: usize,
    pub geometry_entries: usize,
}
impl Default for TransportLimits {
    fn default() -> Self {
        Self {
            buffers: 64,
            buffer_bytes: 128 * 1024 * 1024,
            scenes: 8,
            scene_bytes: 128 * 1024 * 1024,
            geometry_entries: 65536,
        }
    }
}
impl TransportLimits {
    pub(crate) fn validate(self) -> Result<Self, WireError> {
        if self.buffers == 0
            || self.buffers > 1024
            || self.buffer_bytes > 512 * 1024 * 1024
            || self.scenes == 0
            || self.scenes > 64
            || self.scene_bytes > 512 * 1024 * 1024
            || self.geometry_entries == 0
            || self.geometry_entries > 65536
        {
            Err(WireError::capacity())
        } else {
            Ok(self)
        }
    }
}
