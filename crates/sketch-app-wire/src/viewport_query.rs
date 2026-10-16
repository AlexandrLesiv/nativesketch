use crate::{scene_store::Scenes, *};
use sketch_core::{PathCommand, PixelViewport, PreparationLimits, PreparedViewportPath};
pub(crate) struct Prepared {
    id: u64,
    commands: Vec<PathCommand>,
    bytes: usize,
}
impl Scenes {
    pub fn prepare_viewport(
        &mut self,
        reply: &mut Reply,
        scene: u64,
        selector: u32,
        index: u32,
        width: u32,
        height: u32,
    ) -> Result<(), WireError> {
        if self.prepared.is_some() {
            return Err(WireError::plain(9));
        }
        let lease = self.lease(scene)?;
        let stroke = match (selector, index) {
            (0, index) => lease.scene.committed.get(index as usize),
            (1, 0) => lease.scene.live.as_ref(),
            (2, 0) => lease
                .scene
                .transform_preview
                .as_ref()
                .and_then(|p| p.replacement.as_ref()),
            _ => return Err(WireError::malformed()),
        }
        .ok_or(WireError::malformed())?;
        let provenance = [
            scene,
            self.geometry_id(stroke),
            stroke.stroke_id,
            stroke.order as u64,
        ];
        let id = crate::identity::reserve(1)?;
        let available = self.limits.scene_bytes.saturating_sub(self.bytes);
        let prepared = sketch_core::prepare_viewport_path(
            stroke,
            PixelViewport {
                document: lease.viewport,
                pixel_width: width,
                pixel_height: height,
            },
            PreparationLimits {
                bytes: available.min(64 * 1024 * 1024),
                ..Default::default()
            },
        )
        .map_err(AppError::Scene)?;
        let retained = prepared
            .retained_bytes()
            .checked_add(size_of::<Prepared>())
            .ok_or(WireError::capacity())?;
        let (mode, reference, count) = match &prepared {
            PreparedViewportPath::LocalGeometry => (0, 0, 0),
            PreparedViewportPath::PixelCommands(commands) => {
                if retained > available {
                    return Err(WireError::capacity());
                }
                (1, id, commands.len() as u64)
            }
        };
        // Reply reservation happened before preparation; writes cannot allocate or fail.
        reply.u32(mode);
        reply.u32(0);
        reply.u64(reference);
        for value in provenance {
            reply.u64(value);
        }
        reply.u64(count);
        reply.u32(width);
        reply.u32(height);
        if let PreparedViewportPath::PixelCommands(commands) = prepared {
            self.bytes += retained;
            self.prepared = Some(Prepared {
                id,
                commands,
                bytes: retained,
            });
        }
        Ok(())
    }
    pub fn prepared_commands(
        &self,
        reply: &mut Reply,
        id: u64,
        first: u32,
        limit: u32,
    ) -> Result<(), WireError> {
        let p = self
            .prepared
            .as_ref()
            .filter(|p| p.id == id)
            .ok_or(WireError::plain(7))?;
        let first = first as usize;
        if first > p.commands.len() {
            return Err(WireError::malformed());
        }
        let count = (limit.min(744) as usize).min(p.commands.len() - first);
        reply.u32(count as u32);
        reply.u32(0);
        for command in &p.commands[first..first + count] {
            reply.u32(command.verb);
            reply.u32(0);
            for value in command.rgba {
                reply.f64(value);
            }
            for value in command.points {
                reply.f64(value);
            }
        }
        Ok(())
    }
    pub fn release_prepared(&mut self, id: u64) -> Result<(), WireError> {
        if self.prepared.as_ref().is_none_or(|p| p.id != id) {
            return Err(WireError::plain(7));
        }
        let prepared = self.prepared.take().expect("validated prepared path");
        self.bytes -= prepared.bytes;
        Ok(())
    }
}
