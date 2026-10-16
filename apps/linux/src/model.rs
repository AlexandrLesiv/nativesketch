use sketch_core::{Brush, Editor, Input, Sample};

pub(crate) struct Model {
    pub(crate) editor: Editor,
    pub(crate) zoom: f64,
    pub(crate) offset: (f64, f64),
    pub(crate) size: (i32, i32),
    pub(crate) select: bool,
    pub(crate) space: bool,
    pub(crate) brush: Brush,
    pub(crate) sample: u64,
    pub(crate) start_time: u32,
    pub(crate) gesture: Gesture,
}
#[derive(Default)]
pub(crate) enum Gesture {
    #[default]
    Idle,
    Ink,
    Pan(f64, f64),
    Resize(f64, f64, f64),
}
impl Model {
    pub(crate) fn new() -> Self {
        Self {
            editor: Editor::new(),
            zoom: 1.0,
            offset: (0.0, 0.0),
            size: (0, 0),
            select: false,
            space: false,
            brush: Brush {
                rgba: [0.14, 0.39, 0.92, 1.0],
                width: 4.0,
            },
            sample: 0,
            start_time: 0,
            gesture: Gesture::Idle,
        }
    }
    pub(crate) fn cancel(&mut self) {
        self.editor.cancel();
        self.gesture = Gesture::Idle;
    }
    pub(crate) fn point(&self, x: f64, y: f64) -> (f64, f64) {
        (
            (x - self.offset.0) / self.zoom,
            (y - self.offset.1) / self.zoom,
        )
    }
    pub(crate) fn zoom_to(&mut self, value: f64) {
        self.cancel();
        let (x, y) = (self.size.0 as f64 / 2.0, self.size.1 as f64 / 2.0);
        let p = self.point(x, y);
        self.zoom = value.clamp(0.05, 32.0);
        self.offset = (x - p.0 * self.zoom, y - p.1 * self.zoom);
    }
    pub(crate) fn layout(&mut self, w: i32, h: i32) {
        if self.size != (w, h) {
            self.cancel();
            if self.size.0 != 0 {
                self.offset.0 += (w - self.size.0) as f64 / 2.0;
                self.offset.1 += (h - self.size.1) as f64 / 2.0;
            }
            self.size = (w, h);
        }
    }
    pub(crate) fn down(
        &mut self,
        x: f64,
        y: f64,
        pressure: Option<f64>,
        time: u32,
        device: u32,
    ) -> sketch_core::Result<()> {
        self.cancel();
        let p = self.point(x, y);
        if self.space {
            self.gesture = Gesture::Pan(x, y);
            return Ok(());
        }
        if self.select {
            let s = self.editor.state();
            let b = s.bounds;
            if s.selected != 0 {
                let corners = [
                    (b.x, b.y),
                    (b.x + b.width, b.y),
                    (b.x, b.y + b.height),
                    (b.x + b.width, b.y + b.height),
                ];
                if let Some(i) = corners
                    .iter()
                    .position(|c| (p.0 - c.0).hypot(p.1 - c.1) < 14.0 / self.zoom)
                {
                    let a = corners[3 - i];
                    self.editor.resize_begin()?;
                    self.gesture =
                        Gesture::Resize(a.0, a.1, (p.0 - a.0).hypot(p.1 - a.1).max(0.01));
                    return Ok(());
                }
            }
            self.editor.select_at(p.0, p.1, 8.0 / self.zoom)?;
        } else {
            self.editor.begin(1, device, self.brush)?;
            self.sample = 0;
            self.start_time = time;
            self.gesture = Gesture::Ink;
            self.motion(x, y, pressure, time)?;
        }
        Ok(())
    }
    pub(crate) fn motion(
        &mut self,
        x: f64,
        y: f64,
        pressure: Option<f64>,
        time: u32,
    ) -> sketch_core::Result<()> {
        let p = self.point(x, y);
        match self.gesture {
            Gesture::Ink => {
                self.sample += 1;
                self.editor.append(
                    1,
                    &[Input::Actual(Sample {
                        x: p.0,
                        y: p.1,
                        timestamp: time.wrapping_sub(self.start_time) as f64 / 1000.0,
                        pressure,
                        id: self.sample,
                    })],
                )?;
            }
            Gesture::Pan(a, b) => {
                self.offset.0 += x - a;
                self.offset.1 += y - b;
                self.gesture = Gesture::Pan(x, y);
            }
            Gesture::Resize(a, b, d) => {
                self.editor
                    .resize_update(((p.0 - a).hypot(p.1 - b) / d).max(0.01), a, b)?
            }
            Gesture::Idle => {}
        }
        Ok(())
    }
    pub(crate) fn up(&mut self) -> sketch_core::Result<()> {
        match self.gesture {
            Gesture::Ink => self.editor.commit(1)?,
            Gesture::Resize(..) => self.editor.resize_commit()?,
            _ => {}
        }
        self.gesture = Gesture::Idle;
        Ok(())
    }
}
