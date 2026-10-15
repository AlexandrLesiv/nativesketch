use super::{Brush, Sample, Transform};
#[derive(Debug, Clone, PartialEq)]
pub struct Stroke {
    pub id: u64,
    pub device: u32,
    pub pointer_id: u64,
    pub brush: Brush,
    pub samples: Vec<Sample>,
    pub transform: Transform,
}
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Document {
    pub strokes: Vec<Stroke>,
}
