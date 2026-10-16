use crate::{Error, Result};
pub(super) struct Reader<'a> {
    pub(super) data: &'a [u8],
    pub(super) pos: usize,
}
impl Reader<'_> {
    pub(super) fn u64(&mut self) -> Result<u64> {
        let end = self.pos.checked_add(8).ok_or(Error::Format)?;
        let b = self.data.get(self.pos..end).ok_or(Error::Format)?;
        self.pos = end;
        Ok(u64::from_le_bytes(b.try_into().map_err(|_| Error::Format)?))
    }
    pub(super) fn f64(&mut self) -> Result<f64> {
        Ok(f64::from_bits(self.u64()?))
    }
}
