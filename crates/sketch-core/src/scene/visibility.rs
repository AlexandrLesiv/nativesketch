use super::{RenderViewport, SceneError};
use crate::{Bounds, Transform};

impl RenderViewport {
    pub(super) fn validate(self) -> Result<(), SceneError> {
        let values = [self.x, self.y, self.width, self.height, self.margin];
        if values.iter().any(|v| !v.is_finite())
            || self.width <= 0.0
            || self.height <= 0.0
            || self.margin < 0.0
            || ![
                self.x - self.margin,
                self.y - self.margin,
                self.x + self.width + self.margin,
                self.y + self.height + self.margin,
            ]
            .iter()
            .all(|v| v.is_finite())
        {
            return Err(SceneError::InvalidViewport);
        }
        Ok(())
    }
    pub(super) fn intersects(self, bounds: Bounds, transform: Transform) -> bool {
        let x = bounds.x * transform.scale + transform.tx;
        let y = bounds.y * transform.scale + transform.ty;
        x <= self.x + self.width + self.margin
            && y <= self.y + self.height + self.margin
            && x + bounds.width * transform.scale >= self.x - self.margin
            && y + bounds.height * transform.scale >= self.y - self.margin
    }
}
