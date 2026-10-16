use super::{ink::InkSession, transform::TransformSession};
use crate::history::Change;
use crate::{Brush, Document, Error, Input, MAX_STROKES, Result, Stroke, Transform};
/// Exactly one active transaction; real and predicted ink remain separate.
#[derive(Debug, Default)]
pub(crate) enum ActiveInteraction {
    #[default]
    Idle,
    Drawing(InkSession),
    Transforming(TransformSession),
}
impl ActiveInteraction {
    pub(crate) fn idle(&self) -> Result<()> {
        if matches!(self, Self::Idle) {
            Ok(())
        } else {
            Err(Error::InvalidState)
        }
    }
    pub(crate) fn kind(&self) -> u32 {
        match self {
            Self::Idle => 0,
            Self::Drawing(_) => 1,
            Self::Transforming(_) => 2,
        }
    }
    pub(crate) fn transform(&self, index: usize) -> Option<Transform> {
        match self {
            Self::Transforming(r) if r.index == index => Some(r.preview),
            _ => None,
        }
    }
    pub(crate) fn begin(
        &mut self,
        id: u64,
        count: usize,
        pointer: u64,
        device: u32,
        brush: Brush,
    ) -> Result<()> {
        self.idle()?;
        if id == u64::MAX {
            return Err(Error::InvalidState);
        }
        brush.validate()?;
        if device > 2 || count >= MAX_STROKES {
            return Err(Error::InvalidArgument);
        }
        *self = Self::Drawing(InkSession::new(id, pointer, device, brush));
        Ok(())
    }
    pub(crate) fn append(&mut self, pointer: u64, inputs: &[Input]) -> Result<()> {
        match self {
            Self::Drawing(ink) => ink.append(pointer, inputs),
            _ => Err(Error::InvalidState),
        }
    }
    pub(crate) fn commit(&mut self, pointer: u64) -> Result<Stroke> {
        let Self::Drawing(ink) = self else {
            return Err(Error::InvalidState);
        };
        if ink.stroke.pointer_id != pointer || ink.stroke.samples.is_empty() {
            return Err(Error::InvalidState);
        }
        let Self::Drawing(ink) = std::mem::take(self) else {
            unreachable!("checked ink")
        };
        Ok(ink.stroke)
    }
    pub(crate) fn resize_begin(&mut self, document: &Document, index: Option<usize>) -> Result<()> {
        self.idle()?;
        let index = index.ok_or(Error::InvalidState)?;
        *self = Self::Transforming(TransformSession::new(
            index,
            document.strokes[index].transform,
        ));
        Ok(())
    }
    pub(crate) fn resize_update(
        &mut self,
        document: &Document,
        scale: f64,
        x: f64,
        y: f64,
    ) -> Result<()> {
        match self {
            Self::Transforming(r) => r.update(document, scale, x, y),
            _ => Err(Error::InvalidState),
        }
    }
    pub(crate) fn resize_commit(&mut self, document: &mut Document) -> Result<Option<Change>> {
        if !matches!(self, Self::Transforming(_)) {
            return Err(Error::InvalidState);
        }
        let Self::Transforming(r) = std::mem::take(self) else {
            unreachable!("checked transform")
        };
        Ok(r.commit(document))
    }
    pub(crate) fn cancel(&mut self) {
        *self = Self::Idle;
    }
}
