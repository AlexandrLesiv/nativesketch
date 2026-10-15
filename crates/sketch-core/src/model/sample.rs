use super::coordinate;
use crate::{Error, Result};
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sample {
    pub x: f64,
    pub y: f64,
    pub timestamp: f64,
    pub pressure: Option<f64>,
    pub id: u64,
}
impl Sample {
    pub(crate) fn validate(self) -> Result<()> {
        if !coordinate(self.x)
            || self.x.abs() > 1e9
            || !coordinate(self.y)
            || self.y.abs() > 1e9
            || !self.timestamp.is_finite()
            || self.timestamp < 0.0
            || self
                .pressure
                .is_some_and(|p| !p.is_finite() || !(0.0..=1.0).contains(&p))
        {
            return Err(Error::InvalidArgument);
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Copy)]
pub enum Input {
    Actual(Sample),
    Predicted(Sample),
    Correction(Sample),
}
