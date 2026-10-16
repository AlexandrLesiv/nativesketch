use crate::{
    budget::{Budget, Permit},
    *,
};
use std::sync::Arc;
/// Owned immutable response. Dropping releases its transport byte/count reservation.
#[derive(Debug)]
pub struct Reply {
    bytes: Vec<u8>,
    _permit: Permit,
}
impl Reply {
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
    pub(crate) fn reserve(
        budget: &Arc<Budget>,
        opcode: u32,
        request: u64,
        maximum: usize,
    ) -> Result<Self, WireError> {
        let permit = budget.reserve(maximum)?;
        let mut bytes = Vec::new();
        bytes
            .try_reserve_exact(maximum)
            .map_err(|_| WireError::capacity())?;
        bytes.extend_from_slice(&WireError::plain(0).packet(opcode, request));
        Ok(Self {
            bytes,
            _permit: permit,
        })
    }
    pub(crate) fn raw(&mut self, bytes: &[u8]) {
        assert!(self.bytes.len() + bytes.len() <= self.bytes.capacity());
        self.bytes.extend_from_slice(bytes);
    }
    pub(crate) fn u32(&mut self, value: u32) {
        self.raw(&value.to_le_bytes());
    }
    pub(crate) fn u64(&mut self, value: u64) {
        self.raw(&value.to_le_bytes());
    }
    pub(crate) fn f64(&mut self, value: f64) {
        self.raw(&value.to_le_bytes());
    }
    pub(crate) fn zeros(&mut self, count: usize) {
        for _ in 0..count {
            self.raw(&[0]);
        }
    }
    pub(crate) fn finish(mut self) -> Self {
        let size = u32::try_from(self.bytes.len() - 24).expect("bounded response");
        self.bytes[12..16].copy_from_slice(&size.to_le_bytes());
        self
    }
    pub(crate) fn resource(&mut self, resource: Option<ResourceDescriptor>) {
        if let Some(v) = resource {
            self.u32(v.provider.0);
            self.u32(
                u32::from(v.access.read)
                    | (u32::from(v.access.write) << 1)
                    | (u32::from(v.access.confirmed_write) << 2),
            );
            self.u64(v.resource.0);
            self.u64(v.canonical.0);
            self.u64(v.generation);
        } else {
            self.zeros(32);
        }
    }
}
