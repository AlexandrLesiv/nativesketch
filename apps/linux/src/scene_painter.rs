use gtk::cairo;
use sketch_core::{RenderScene, SceneStroke};
pub(crate) fn paint(cr: &cairo::Context, scene: &RenderScene) -> Result<(), cairo::Error> {
    let preview = scene.transform_preview.as_ref();
    let mut inserted = false;
    for stroke in scene.committed.iter() {
        if let Some(preview) = preview {
            if !inserted && stroke.order >= preview.order {
                if let Some(replacement) = &preview.replacement {
                    paint_stroke(cr, replacement)?;
                }
                inserted = true;
            }
            if stroke.stroke_id == preview.stroke_id {
                continue;
            }
        }
        paint_stroke(cr, stroke)?;
    }
    if !inserted && let Some(replacement) = preview.and_then(|preview| preview.replacement.as_ref())
    {
        paint_stroke(cr, replacement)?;
    }
    if let Some(live) = &scene.live {
        paint_stroke(cr, live)?;
    }
    Ok(())
}
fn paint_stroke(cr: &cairo::Context, stroke: &SceneStroke) -> Result<(), cairo::Error> {
    cr.save()?;
    let transform = stroke.transform;
    cr.translate(transform.tx, transform.ty);
    cr.scale(transform.scale, transform.scale);
    // Cairo keeps f64 coordinates; geometry commands are original local coordinates.
    cr.new_path();
    for command in stroke.geometry.commands.iter() {
        let c = command.rgba;
        cr.set_source_rgba(c[0], c[1], c[2], c[3]);
        let p = command.points;
        match command.verb {
            0 => cr.move_to(p[0], p[1]),
            1 => cr.line_to(p[0], p[1]),
            2 => cr.curve_to(p[0], p[1], p[2], p[3], p[4], p[5]),
            3 => cr.close_path(),
            _ => {}
        }
    }
    let result = cr.fill();
    cr.restore()?;
    result
}
