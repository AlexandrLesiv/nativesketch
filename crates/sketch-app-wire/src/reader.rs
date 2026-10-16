use crate::WireError;
pub(crate) struct Reader<'a> {
    bytes: &'a [u8],
    offset: usize,
}
impl<'a> Reader<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }
    pub fn take<const N: usize>(&mut self) -> Result<[u8; N], WireError> {
        let end = self.offset.checked_add(N).ok_or(WireError::malformed())?;
        let slice = self
            .bytes
            .get(self.offset..end)
            .ok_or(WireError::malformed())?;
        self.offset = end;
        Ok(slice.try_into().expect("checked length"))
    }
    pub fn u32(&mut self) -> Result<u32, WireError> {
        Ok(u32::from_le_bytes(self.take()?))
    }
    pub fn u64(&mut self) -> Result<u64, WireError> {
        Ok(u64::from_le_bytes(self.take()?))
    }
    pub fn usize(&mut self) -> Result<usize, WireError> {
        usize::try_from(self.u64()?).map_err(|_| WireError::capacity())
    }
    pub fn f64(&mut self) -> Result<f64, WireError> {
        let value = f64::from_le_bytes(self.take()?);
        if value.is_finite() {
            Ok(value)
        } else {
            Err(WireError::malformed())
        }
    }
    pub fn zero(&mut self) -> Result<(), WireError> {
        if self.u32()? == 0 {
            Ok(())
        } else {
            Err(WireError::transport(5))
        }
    }
    pub fn finish(self) -> Result<(), WireError> {
        if self.offset == self.bytes.len() {
            Ok(())
        } else {
            Err(WireError::malformed())
        }
    }
}
