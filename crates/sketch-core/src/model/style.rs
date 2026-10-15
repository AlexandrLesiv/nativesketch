use crate::{Error, Result};
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Brush {
    pub rgba: [f64; 4],
    pub width: f64,
}
impl Brush {
    pub fn validate(self) -> Result<()> {
        if self
            .rgba
            .iter()
            .any(|x| !x.is_finite() || !(0.0..=1.0).contains(x))
            || !self.width.is_finite()
            || !(0.01..=10_000.0).contains(&self.width)
        {
            return Err(Error::InvalidArgument);
        }
        Ok(())
    }
}
