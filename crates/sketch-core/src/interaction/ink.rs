use super::batch;
use crate::{Brush, Error, Input, Result, Sample, Stroke, Transform};
#[derive(Debug)]
pub(crate) struct InkSession {
    pub(crate) stroke: Stroke,
    pub(crate) predicted: Vec<Sample>,
}
impl InkSession {
    pub(super) fn new(id: u64, pointer_id: u64, device: u32, brush: Brush) -> Self {
        Self {
            stroke: Stroke {
                id,
                device,
                pointer_id,
                brush,
                samples: Vec::new(),
                transform: Transform::default(),
            },
            predicted: Vec::new(),
        }
    }
    pub(super) fn append(&mut self, pointer: u64, inputs: &[Input]) -> Result<()> {
        if self.stroke.pointer_id != pointer {
            return Err(Error::InvalidState);
        }
        batch::validate(&self.stroke.samples, inputs)?;
        self.predicted.clear();
        for input in inputs {
            match input {
                Input::Actual(s) => self.stroke.samples.push(*s),
                Input::Predicted(s) => self.predicted.push(*s),
                Input::Correction(s) => {
                    let i = self
                        .stroke
                        .samples
                        .binary_search_by_key(&s.id, |p| p.id)
                        .expect("validated correction");
                    self.stroke.samples[i] = *s;
                }
            }
        }
        Ok(())
    }
}
