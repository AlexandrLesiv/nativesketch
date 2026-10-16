use crate::geometry::hit;
use crate::model::coordinate;
use crate::{Document, Error, Result};
#[derive(Debug, Default)]
pub(super) struct Selection {
    selected: Option<u64>,
}
impl Selection {
    pub(super) fn id(&self) -> u64 {
        self.selected.unwrap_or(0)
    }
    pub(super) fn index(&self, document: &Document) -> Option<usize> {
        document
            .strokes
            .iter()
            .position(|s| Some(s.id) == self.selected)
    }
    pub(super) fn reconcile(&mut self, document: &Document) {
        if self.index(document).is_none() {
            self.selected = None;
        }
    }
    pub(crate) fn select_at(
        &mut self,
        document: &Document,
        x: f64,
        y: f64,
        tolerance: f64,
    ) -> Result<()> {
        if !coordinate(x) || !coordinate(y) || !tolerance.is_finite() || tolerance < 0.0 {
            return Err(Error::InvalidArgument);
        }
        self.selected = document
            .strokes
            .iter()
            .rev()
            .find(|s| hit(s, x, y, tolerance))
            .map(|s| s.id);
        Ok(())
    }
    pub(crate) fn select_cycle(&mut self, document: &Document, direction: i32) -> Result<()> {
        if direction != 1 && direction != -1 {
            return Err(Error::InvalidArgument);
        }
        let n = document.strokes.len();
        if n == 0 {
            self.selected = None;
            return Ok(());
        }
        let i = match self.index(document) {
            Some(i) => (i as i64 + i64::from(direction)).rem_euclid(n as i64) as usize,
            None => {
                if direction == 1 {
                    0
                } else {
                    n - 1
                }
            }
        };
        self.selected = Some(document.strokes[i].id);
        Ok(())
    }
    pub(crate) fn clear(&mut self) {
        self.selected = None;
    }
}
