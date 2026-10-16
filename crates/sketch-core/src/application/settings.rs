use crate::{Brush, Result};
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolId {
    Pen,
    Select,
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BrushSettings(Brush);
impl TryFrom<Brush> for BrushSettings {
    type Error = crate::Error;
    fn try_from(brush: Brush) -> Result<Self> {
        brush.validate()?;
        Ok(Self(brush))
    }
}
impl BrushSettings {
    pub fn brush(self) -> Brush {
        self.0
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceKind {
    Mouse,
    Pen,
    Touch,
}
impl DeviceKind {
    pub(super) fn code(self) -> u32 {
        match self {
            Self::Mouse => 0,
            Self::Pen => 1,
            Self::Touch => 2,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PointerId(pub u64);
