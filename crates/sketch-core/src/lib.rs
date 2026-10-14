//! Renderer-independent editable strokes and transactional editor state.
#![forbid(unsafe_code)]

mod persistence;
#[cfg(test)]
mod tests;

use std::path::Path;

pub const MAX_SAMPLES: usize = 100_000;
pub const MAX_STROKES: usize = 10_000;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    InvalidArgument,
    InvalidState,
    Io,
    Format,
}
pub type Result<T> = std::result::Result<T, Error>;
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sample {
    pub x: f64,
    pub y: f64,
    pub timestamp: f64,
    pub pressure: Option<f64>,
    pub id: u64,
}
impl Sample {
    fn validate(self) -> Result<()> {
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
fn coordinate(x: f64) -> bool {
    x.is_finite() && x.abs() <= 1e12
}
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
    fn validate(self) -> Result<()> {
        if !(1e-6..=1e6).contains(&self.scale) || !coordinate(self.tx) || !coordinate(self.ty) {
            Err(Error::InvalidArgument)
        } else {
            Ok(())
        }
    }
    fn point(self, x: f64, y: f64) -> (f64, f64) {
        (x * self.scale + self.tx, y * self.scale + self.ty)
    }
}
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
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Bounds {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}
#[derive(Debug, Clone, Copy)]
pub enum Input {
    Actual(Sample),
    Predicted(Sample),
    Correction(Sample),
}
#[derive(Debug)]
struct Ink {
    stroke: Stroke,
    predicted: Vec<Sample>,
}
#[derive(Debug, Clone)]
enum Change {
    Insert(usize, Stroke),
    Delete(usize, Stroke),
    Transform(usize, Transform, Transform),
}
#[derive(Debug, Clone)]
struct History {
    change: Change,
    before: u64,
    after: u64,
}
#[derive(Debug)]
struct Resize {
    index: usize,
    original: Transform,
    preview: Transform,
}
#[derive(Debug, Default)]
pub struct Editor {
    document: Document,
    ink: Option<Ink>,
    resize: Option<Resize>,
    selected: Option<u64>,
    undo: Vec<History>,
    redo: Vec<History>,
    revision: u64,
    saved_revision: u64,
    next_revision: u64,
    next_id: u64,
}
#[derive(Debug, Clone, Copy)]
pub struct State {
    pub count: usize,
    pub selected: u64,
    pub can_undo: bool,
    pub can_redo: bool,
    pub dirty: bool,
    pub interaction: u32,
    pub bounds: Bounds,
}
#[derive(Debug, Clone, Copy)]
pub struct PathCommand {
    pub stroke_id: u64,
    pub rgba: [f64; 4],
    pub verb: u32,
    pub points: [f64; 6],
}
impl Editor {
    pub fn new() -> Self {
        Self {
            next_id: 1,
            next_revision: 1,
            ..Self::default()
        }
    }
    pub fn document(&self) -> &Document {
        &self.document
    }
    fn idle(&self) -> Result<()> {
        if self.ink.is_some() || self.resize.is_some() {
            Err(Error::InvalidState)
        } else {
            Ok(())
        }
    }
    fn selected_index(&self) -> Option<usize> {
        self.document
            .strokes
            .iter()
            .position(|s| Some(s.id) == self.selected)
    }
    pub fn state(&self) -> State {
        let bounds = self
            .selected_index()
            .map(|i| {
                let s = &self.document.strokes[i];
                bounds(
                    s,
                    self.resize
                        .as_ref()
                        .filter(|r| r.index == i)
                        .map_or(s.transform, |r| r.preview),
                )
            })
            .unwrap_or_default();
        State {
            count: self.document.strokes.len(),
            selected: self.selected.unwrap_or(0),
            can_undo: !self.undo.is_empty(),
            can_redo: !self.redo.is_empty(),
            dirty: self.revision != self.saved_revision,
            interaction: if self.ink.is_some() {
                1
            } else if self.resize.is_some() {
                2
            } else {
                0
            },
            bounds,
        }
    }
    pub fn begin(&mut self, pointer_id: u64, device: u32, brush: Brush) -> Result<()> {
        self.idle()?;
        brush.validate()?;
        if device > 2 || self.document.strokes.len() >= MAX_STROKES {
            return Err(Error::InvalidArgument);
        }
        self.ink = Some(Ink {
            stroke: Stroke {
                id: self.next_id,
                device,
                pointer_id,
                brush,
                samples: Vec::new(),
                transform: Transform::default(),
            },
            predicted: Vec::new(),
        });
        Ok(())
    }
    pub fn append(&mut self, pointer: u64, inputs: &[Input]) -> Result<()> {
        let ink = self.ink.as_mut().ok_or(Error::InvalidState)?;
        if ink.stroke.pointer_id != pointer {
            return Err(Error::InvalidState);
        }
        if inputs.len() > MAX_SAMPLES
            || ink.stroke.samples.len().saturating_add(
                inputs
                    .iter()
                    .filter(|x| matches!(x, Input::Actual(_)))
                    .count(),
            ) > MAX_SAMPLES
        {
            return Err(Error::InvalidArgument);
        }
        // Validate the entire batch before changing even transient state. Actual ids
        // strictly increase, allowing correction lookup without copying ink history.
        let mut last = ink.stroke.samples.last().copied();
        let mut prediction_started = false;
        for input in inputs {
            let s = match input {
                Input::Actual(s) | Input::Predicted(s) | Input::Correction(s) => *s,
            };
            s.validate()?;
            match input {
                Input::Actual(_) => {
                    if prediction_started
                        || last.is_some_and(|p| s.id <= p.id || s.timestamp < p.timestamp)
                    {
                        return Err(Error::InvalidArgument);
                    }
                    last = Some(s);
                }
                Input::Predicted(_) => {
                    prediction_started = true;
                    if last.is_some_and(|p| s.timestamp < p.timestamp) {
                        return Err(Error::InvalidArgument);
                    }
                }
                Input::Correction(_) => {
                    let i = ink
                        .stroke
                        .samples
                        .binary_search_by_key(&s.id, |p| p.id)
                        .map_err(|_| Error::InvalidArgument)?;
                    if s.timestamp != ink.stroke.samples[i].timestamp {
                        return Err(Error::InvalidArgument);
                    }
                }
            }
        }
        ink.predicted.clear();
        for input in inputs {
            match input {
                Input::Actual(s) => ink.stroke.samples.push(*s),
                Input::Predicted(s) => ink.predicted.push(*s),
                Input::Correction(s) => {
                    let i = ink
                        .stroke
                        .samples
                        .binary_search_by_key(&s.id, |p| p.id)
                        .expect("validated correction");
                    ink.stroke.samples[i] = *s;
                }
            }
        }
        Ok(())
    }
    pub fn commit(&mut self, pointer: u64) -> Result<()> {
        let ink = self.ink.as_ref().ok_or(Error::InvalidState)?;
        if ink.stroke.pointer_id != pointer || ink.stroke.samples.is_empty() {
            return Err(Error::InvalidState);
        }
        let stroke = self.ink.take().expect("checked ink").stroke;
        self.next_id = self.next_id.checked_add(1).ok_or(Error::InvalidState)?;
        let index = self.document.strokes.len();
        self.document.strokes.push(stroke.clone());
        self.record(Change::Insert(index, stroke));
        Ok(())
    }
    pub fn cancel(&mut self) {
        self.ink = None;
        self.resize = None;
    }
    fn record(&mut self, change: Change) {
        let after = self.next_revision;
        self.next_revision += 1;
        self.undo.push(History {
            change,
            before: self.revision,
            after,
        });
        self.revision = after;
        self.redo.clear();
    }
    pub fn select_at(&mut self, x: f64, y: f64, tolerance: f64) -> Result<()> {
        self.idle()?;
        if !coordinate(x) || !coordinate(y) || !tolerance.is_finite() || tolerance < 0.0 {
            return Err(Error::InvalidArgument);
        }
        self.selected = self
            .document
            .strokes
            .iter()
            .rev()
            .find(|s| hit(s, x, y, tolerance))
            .map(|s| s.id);
        Ok(())
    }
    pub fn select_cycle(&mut self, direction: i32) -> Result<()> {
        self.idle()?;
        if direction != 1 && direction != -1 {
            return Err(Error::InvalidArgument);
        }
        let n = self.document.strokes.len();
        if n == 0 {
            self.selected = None;
            return Ok(());
        }
        let i = match self.selected_index() {
            Some(i) => (i as i64 + i64::from(direction)).rem_euclid(n as i64) as usize,
            None => {
                if direction == 1 {
                    0
                } else {
                    n - 1
                }
            }
        };
        self.selected = Some(self.document.strokes[i].id);
        Ok(())
    }
    pub fn select_clear(&mut self) -> Result<()> {
        self.idle()?;
        self.selected = None;
        Ok(())
    }
    pub fn delete_selected(&mut self) -> Result<()> {
        self.idle()?;
        let i = self.selected_index().ok_or(Error::InvalidState)?;
        let s = self.document.strokes.remove(i);
        self.selected = None;
        self.record(Change::Delete(i, s));
        Ok(())
    }
    pub fn resize_begin(&mut self) -> Result<()> {
        self.idle()?;
        let index = self.selected_index().ok_or(Error::InvalidState)?;
        let original = self.document.strokes[index].transform;
        self.resize = Some(Resize {
            index,
            original,
            preview: original,
        });
        Ok(())
    }
    pub fn resize_update(&mut self, scale: f64, x: f64, y: f64) -> Result<()> {
        let r = self.resize.as_mut().ok_or(Error::InvalidState)?;
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
        let b = bounds(&self.document.strokes[r.index], preview);
        if !coordinate(b.x) || !coordinate(b.y) || !coordinate(b.width) || !coordinate(b.height) {
            return Err(Error::InvalidArgument);
        }
        r.preview = preview;
        Ok(())
    }
    pub fn resize_commit(&mut self) -> Result<()> {
        let r = self.resize.take().ok_or(Error::InvalidState)?;
        if r.preview != r.original {
            self.document.strokes[r.index].transform = r.preview;
            self.record(Change::Transform(r.index, r.original, r.preview));
        }
        Ok(())
    }
    fn apply(&mut self, change: &Change, forward: bool) {
        match (change, forward) {
            (Change::Insert(i, s), true) | (Change::Delete(i, s), false) => {
                self.document.strokes.insert(*i, s.clone())
            }
            (Change::Insert(i, _), false) | (Change::Delete(i, _), true) => {
                self.document.strokes.remove(*i);
            }
            (Change::Transform(i, a, b), _) => {
                self.document.strokes[*i].transform = if forward { *b } else { *a }
            }
        }
        if self.selected_index().is_none() {
            self.selected = None;
        }
    }
    pub fn undo(&mut self) -> Result<()> {
        self.idle()?;
        let h = self.undo.pop().ok_or(Error::InvalidState)?;
        self.apply(&h.change, false);
        self.revision = h.before;
        self.redo.push(h);
        Ok(())
    }
    pub fn redo(&mut self) -> Result<()> {
        self.idle()?;
        let h = self.redo.pop().ok_or(Error::InvalidState)?;
        self.apply(&h.change, true);
        self.revision = h.after;
        self.undo.push(h);
        Ok(())
    }
    pub fn clear(&mut self) -> Result<()> {
        self.idle()?;
        *self = Self::new();
        Ok(())
    }
    pub fn save(&mut self, path: &Path) -> Result<()> {
        self.idle()?;
        persistence::save(&self.document, path)?;
        self.saved_revision = self.revision;
        Ok(())
    }
    pub fn load(&mut self, path: &Path) -> Result<()> {
        self.idle()?;
        let document = persistence::load(path)?;
        let next_id = document
            .strokes
            .iter()
            .map(|s| s.id)
            .max()
            .unwrap_or(0)
            .checked_add(1)
            .ok_or(Error::Format)?;
        *self = Self {
            document,
            next_id,
            ..Self::new()
        };
        Ok(())
    }
    pub fn render(&self) -> Vec<PathCommand> {
        let mut out = Vec::new();
        for (i, s) in self.document.strokes.iter().enumerate() {
            let t = self
                .resize
                .as_ref()
                .filter(|r| r.index == i)
                .map_or(s.transform, |r| r.preview);
            render_stroke(s, t, s.samples.iter().copied(), &mut out);
        }
        if let Some(ink) = &self.ink {
            render_stroke(
                &ink.stroke,
                ink.stroke.transform,
                ink.stroke.samples.iter().chain(&ink.predicted).copied(),
                &mut out,
            );
        }
        out
    }
}
fn radius(s: &Stroke, p: Sample) -> f64 {
    s.brush.width * 0.5 * p.pressure.map_or(1.0, |p| 0.2 + 0.8 * p)
}
fn bounds(s: &Stroke, t: Transform) -> Bounds {
    let mut low = [f64::INFINITY; 2];
    let mut high = [f64::NEG_INFINITY; 2];
    for p in &s.samples {
        let (x, y) = t.point(p.x, p.y);
        let r = radius(s, *p) * t.scale;
        low[0] = low[0].min(x - r);
        low[1] = low[1].min(y - r);
        high[0] = high[0].max(x + r);
        high[1] = high[1].max(y + r);
    }
    Bounds {
        x: low[0],
        y: low[1],
        width: high[0] - low[0],
        height: high[1] - low[1],
    }
}
fn hit(s: &Stroke, x: f64, y: f64, tolerance: f64) -> bool {
    let t = s.transform;
    let x = (x - t.tx) / t.scale;
    let y = (y - t.ty) / t.scale;
    let tol = tolerance / t.scale;
    if s.samples
        .iter()
        .any(|p| (x - p.x).hypot(y - p.y) <= radius(s, *p) + tol)
    {
        return true;
    }
    s.samples.windows(2).any(|w| {
        let a = w[0];
        let b = w[1];
        let dx = b.x - a.x;
        let dy = b.y - a.y;
        let d = dx * dx + dy * dy;
        if d == 0.0 {
            return false;
        }
        let u = (((x - a.x) * dx + (y - a.y) * dy) / d).clamp(0.0, 1.0);
        (x - a.x - u * dx).hypot(y - a.y - u * dy)
            <= radius(s, a) + (radius(s, b) - radius(s, a)) * u + tol
    })
}
fn render_stroke(
    s: &Stroke,
    t: Transform,
    samples: impl Iterator<Item = Sample>,
    out: &mut Vec<PathCommand>,
) {
    let mut previous = None;
    let mut emit = |verb, points| {
        out.push(PathCommand {
            stroke_id: s.id,
            rgba: s.brush.rgba,
            verb,
            points,
        })
    };
    for sample in samples {
        let (x, y) = t.point(sample.x, sample.y);
        let r = radius(s, sample) * t.scale;
        if let Some((px, py, pr)) = previous {
            let dx: f64 = x - px;
            let dy: f64 = y - py;
            let len = dx.hypot(dy);
            if len > 0.0 {
                let nx = dy / len;
                let ny = -dx / len;
                emit(0, [px + nx * pr, py + ny * pr, 0.0, 0.0, 0.0, 0.0]);
                emit(1, [x + nx * r, y + ny * r, 0.0, 0.0, 0.0, 0.0]);
                emit(1, [x - nx * r, y - ny * r, 0.0, 0.0, 0.0, 0.0]);
                emit(1, [px - nx * pr, py - ny * pr, 0.0, 0.0, 0.0, 0.0]);
                emit(3, [0.0; 6]);
            }
        }
        // Clockwise circle and segment subpaths use matching winding. The native
        // painter fills their union once per stroke, retaining transparent brush alpha.
        let k = r * 0.552_284_749_830_793_6;
        emit(0, [x + r, y, 0.0, 0.0, 0.0, 0.0]);
        emit(2, [x + r, y + k, x + k, y + r, x, y + r]);
        emit(2, [x - k, y + r, x - r, y + k, x - r, y]);
        emit(2, [x - r, y - k, x - k, y - r, x, y - r]);
        emit(2, [x + k, y - r, x + r, y - k, x + r, y]);
        emit(3, [0.0; 6]);
        previous = Some((x, y, r));
    }
}
