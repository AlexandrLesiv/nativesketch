use crate::{Error, Result};
pub(crate) fn coordinate(x: f64) -> bool {
    x.is_finite() && x.abs() <= 1e12
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform {
    pub scale: f64,
    pub tx: f64,
    pub ty: f64,
}
impl Default for Transform {
    fn default() -> Self {
        Self {
            scale: 1.0,
            tx: 0.0,
            ty: 0.0,
        }
    }
}
impl Transform {
    pub(crate) fn validate(self) -> Result<()> {
        if !(1e-6..=1e6).contains(&self.scale) || !coordinate(self.tx) || !coordinate(self.ty) {
            Err(Error::InvalidArgument)
        } else {
            Ok(())
        }
    }
    pub(crate) fn point(self, x: f64, y: f64) -> (f64, f64) {
        (x * self.scale + self.tx, y * self.scale + self.ty)
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Bounds {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}
