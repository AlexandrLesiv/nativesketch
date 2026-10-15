use crate::geometry::bounds;
use crate::history::Change;
use crate::model::coordinate;
use crate::{Document, Error, Result, Transform};
#[derive(Debug)]
pub(crate) struct TransformSession {
    pub(crate) index: usize,
    original: Transform,
    pub(crate) preview: Transform,
}
impl TransformSession {
    pub(super) fn new(index: usize, original: Transform) -> Self {
        Self {
            index,
            original,
            preview: original,
        }
    }
    pub(super) fn update(&mut self, document: &Document, scale: f64, x: f64, y: f64) -> Result<()> {
        let r = self;
        if !scale.is_finite() || scale <= 0.0 || !coordinate(x) || !coordinate(y) {
            return Err(Error::InvalidArgument);
        }
        let preview = if scale == 1.0 {
            r.original
        } else {
            Transform {
                scale: r.original.scale * scale,
                tx: x + (r.original.tx - x) * scale,
                ty: y + (r.original.ty - y) * scale,
            }
        };
        preview.validate()?;
        let b = bounds(&document.strokes[r.index], preview);
        if !coordinate(b.x) || !coordinate(b.y) || !coordinate(b.width) || !coordinate(b.height) {
            return Err(Error::InvalidArgument);
        }
        r.preview = preview;
        Ok(())
    }

    pub(super) fn commit(self, document: &mut Document) -> Option<Change> {
        if self.preview == self.original {
            return None;
        }
        document.strokes[self.index].transform = self.preview;
        Some(Change::Transform(self.index, self.original, self.preview))
    }
}
