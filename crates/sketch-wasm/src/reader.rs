use sketch_core::Error;
pub(crate) struct Reader<'a>(pub(crate) &'a [u8]);
impl Reader<'_> {
    pub(crate) fn take<const N: usize>(&mut self) -> Result<[u8; N], Error> {
        let bytes = self.0.get(..N).ok_or(Error::InvalidArgument)?;
        let value = bytes.try_into().map_err(|_| Error::InvalidArgument)?;
        self.0 = &self.0[N..];
        Ok(value)
    }
    pub(crate) fn u64(&mut self) -> Result<u64, Error> {
        Ok(u64::from_le_bytes(self.take()?))
    }
    pub(crate) fn u32(&mut self) -> Result<u32, Error> {
        Ok(u32::from_le_bytes(self.take()?))
    }
    pub(crate) fn f64(&mut self) -> Result<f64, Error> {
        Ok(f64::from_le_bytes(self.take()?))
    }
    pub(crate) fn done(&self) -> Result<(), Error> {
        if self.0.is_empty() {
            Ok(())
        } else {
            Err(Error::InvalidArgument)
        }
    }
}
