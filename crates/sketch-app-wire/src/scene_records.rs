use crate::{scene_store::*, *};
use sketch_core::SceneStroke;
impl Scenes {
    pub fn info(&self, reply: &mut Reply, id: u64, unchanged: bool) -> Result<(), WireError> {
        let l = self.lease(id)?;
        let s = &l.scene;
        for v in [
            l.id,
            s.key.session.get(),
            s.key.revision.0,
            s.key.generation.get(),
            l.root,
            s.committed.len() as u64,
        ] {
            reply.u64(v);
        }
        let replacement = s
            .transform_preview
            .as_ref()
            .and_then(|p| p.replacement.as_ref());
        for v in [
            unchanged,
            s.live.is_some(),
            s.transform_preview.is_some(),
            replacement.is_some(),
        ] {
            reply.u32(u32::from(v));
        }
        let v = l.viewport;
        for x in [v.x, v.y, v.width, v.height, v.margin] {
            reply.f64(x);
        }
        reply.u64(s.transform_preview.as_ref().map_or(0, |p| p.stroke_id));
        reply.u64(s.transform_preview.as_ref().map_or(0, |p| p.order as u64));
        self.stroke(reply, s.live.as_ref());
        self.stroke(reply, replacement);
        Ok(())
    }
    pub fn stroke(&self, reply: &mut Reply, stroke: Option<&SceneStroke>) {
        if let Some(s) = stroke {
            reply.u64(s.stroke_id);
            reply.u64(s.order as u64);
            reply.u64(self.geometry_id(s));
            reply.f64(s.transform.scale);
            reply.f64(s.transform.tx);
            reply.f64(s.transform.ty);
        } else {
            reply.zeros(48);
        }
    }
    pub fn descriptors(
        &self,
        reply: &mut Reply,
        id: u64,
        first: u32,
        limit: u32,
    ) -> Result<(), WireError> {
        let strokes = &self.lease(id)?.scene.committed;
        let range = page(strokes.len(), first, limit, 1365)?;
        reply.u32(range.len() as u32);
        reply.u32(0);
        for s in &strokes[range] {
            self.stroke(reply, Some(s));
        }
        Ok(())
    }
    pub fn geometry_info(&self, reply: &mut Reply, scene: u64, id: u64) -> Result<(), WireError> {
        let g = self.geometry(scene, id)?;
        let b = g.bounds;
        for v in [g.origin[0], g.origin[1], b.x, b.y, b.width, b.height] {
            reply.f64(v);
        }
        reply.u64(g.commands.len() as u64);
        Ok(())
    }
    pub fn commands(
        &self,
        reply: &mut Reply,
        scene: u64,
        id: u64,
        first: u32,
        limit: u32,
    ) -> Result<(), WireError> {
        let g = self.geometry(scene, id)?;
        let range = page(g.commands.len(), first, limit, 744)?;
        reply.u32(range.len() as u32);
        reply.u32(0);
        for p in &g.commands[range] {
            let verb = match p.verb {
                0 => 0,
                1 => 1,
                2 => 2,
                3 => 3,
                _ => return Err(WireError::malformed()),
            };
            reply.u32(verb);
            reply.u32(0);
            for v in p.rgba {
                reply.f64(v);
            }
            for v in p.points {
                reply.f64(v);
            }
        }
        Ok(())
    }
}
fn page(
    count: usize,
    first: u32,
    limit: u32,
    maximum: u32,
) -> Result<std::ops::Range<usize>, WireError> {
    let first = usize::try_from(first).map_err(|_| WireError::malformed())?;
    if first > count {
        return Err(WireError::malformed());
    }
    let limit = usize::try_from(limit.min(maximum)).expect("bounded page");
    Ok(first..first + limit.min(count - first))
}
