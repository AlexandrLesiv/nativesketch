use crate::command_owner::CommandOwner;
use sketch_app::Intent;
use sketch_core::application::*;

pub(crate) struct Model {
    pub(crate) owner: CommandOwner,
    pub(crate) zoom: f64,
    pub(crate) offset: (f64, f64),
    pub(crate) size: (i32, i32),
    pub(crate) space: bool,
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
            owner: CommandOwner::new(),
            zoom: 1.0,
            offset: (0.0, 0.0),
            size: (0, 0),
            space: false,
            sample: 0,
            start_time: 0,
            gesture: Gesture::Idle,
        }
    }
    pub(crate) fn cancel(&mut self) {
        if let Err(error) = self
            .owner
            .dispatch(Intent::Edit(EditorCommand::CancelInteraction))
        {
            crate::diagnostics::record(format_args!("event=cancel_rejected error={error:?}"));
        }
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
}
