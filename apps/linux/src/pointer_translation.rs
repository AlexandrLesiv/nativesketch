use crate::model::{Gesture, Model};
use sketch_app::{AppError, Intent};
use sketch_core::application::*;
use sketch_core::{Input, Sample};
impl Model {
    pub(crate) fn down(
        &mut self,
        x: f64,
        y: f64,
        pressure: Option<f64>,
        time: u32,
        device: u32,
    ) -> Result<(), AppError> {
        self.cancel();
        let p = self.point(x, y);
        if self.space {
            self.gesture = Gesture::Pan(x, y);
            return Ok(());
        }
        if self.owner.app.projection(self.owner.session)?.editor.tool == ToolId::Select {
            let s = self.owner.state();
            let b = self.owner.app.selected_bounds(self.owner.session)?;
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
                    self.owner
                        .dispatch(Intent::Edit(EditorCommand::BeginTransform))?;
                    self.gesture =
                        Gesture::Resize(a.0, a.1, (p.0 - a.0).hypot(p.1 - a.1).max(0.01));
                    return Ok(());
                }
            }
            self.owner.dispatch(Intent::Edit(EditorCommand::SelectAt {
                x: p.0,
                y: p.1,
                tolerance: 8.0 / self.zoom,
            }))?;
        } else {
            self.owner
                .dispatch(Intent::Edit(EditorCommand::BeginStroke {
                    pointer: PointerId(1),
                    device: match device {
                        1 => DeviceKind::Pen,
                        2 => DeviceKind::Touch,
                        _ => DeviceKind::Mouse,
                    },
                }))?;
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
    ) -> Result<(), AppError> {
        let p = self.point(x, y);
        match self.gesture {
            Gesture::Ink => {
                self.sample += 1;
                self.owner.dispatch(Intent::Input(SampleBatch {
                    pointer: PointerId(1),
                    samples: &[Input::Actual(Sample {
                        x: p.0,
                        y: p.1,
                        timestamp: time.wrapping_sub(self.start_time) as f64 / 1000.0,
                        pressure,
                        id: self.sample,
                    })],
                }))?;
            }
            Gesture::Pan(a, b) => {
                self.offset.0 += x - a;
                self.offset.1 += y - b;
                self.gesture = Gesture::Pan(x, y);
            }
            Gesture::Resize(a, b, d) => {
                self.owner
                    .dispatch(Intent::Edit(EditorCommand::UpdateTransform {
                        scale: ((p.0 - a).hypot(p.1 - b) / d).max(0.01),
                        anchor_x: a,
                        anchor_y: b,
                    }))?
            }
            Gesture::Idle => {}
        }
        Ok(())
    }
    pub(crate) fn up(&mut self) -> Result<(), AppError> {
        match self.gesture {
            Gesture::Ink => self
                .owner
                .dispatch(Intent::Edit(EditorCommand::CommitStroke(PointerId(1))))?,
            Gesture::Resize(..) => self
                .owner
                .dispatch(Intent::Edit(EditorCommand::CommitTransform))?,
            _ => {}
        }
        self.gesture = Gesture::Idle;
        Ok(())
    }
}
