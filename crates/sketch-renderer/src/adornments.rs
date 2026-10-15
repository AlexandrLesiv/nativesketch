use crate::RenderRequest;
use sketch_core::Bounds;
use skia_safe::{Canvas, Color4f, Paint, Rect, paint::Style};
fn paint(color: [f64; 4], width: f32) -> Paint {
    let mut value = Paint::new(
        Color4f::new(
            color[0] as f32,
            color[1] as f32,
            color[2] as f32,
            color[3] as f32,
        ),
        None,
    );
    value
        .set_anti_alias(true)
        .set_style(Style::Stroke)
        .set_stroke_width(width);
    value
}
pub(crate) fn draw(canvas: &Canvas, request: &RenderRequest, selected: Option<Bounds>) {
    canvas.save();
    canvas.scale(request.scale());
    if let Some(b) = selected {
        let v = request.viewport();
        let x = ((b.x - v.x) * request.zoom) as f32;
        let y = ((b.y - v.y) * request.zoom) as f32;
        let right = ((b.x + b.width - v.x) * request.zoom) as f32;
        let bottom = ((b.y + b.height - v.y) * request.zoom) as f32;
        let outline = paint(request.selection_rgba, 1.5);
        canvas.draw_rect(Rect::new(x, y, right, bottom), &outline);
        let mut fill = Paint::default();
        fill.set_color(skia_safe::Color::WHITE).set_anti_alias(true);
        let handle = paint(request.selection_rgba, 1.0);
        for (px, py) in [(x, y), (right, y), (right, bottom), (x, bottom)] {
            let rect = Rect::from_xywh(px - 4.0, py - 4.0, 8.0, 8.0);
            canvas.draw_rect(rect, &fill);
            canvas.draw_rect(rect, &handle);
        }
    }
    if request.focused != 0 {
        canvas.draw_rect(
            Rect::from_xywh(
                2.0,
                2.0,
                request.logical_width as f32 - 4.0,
                request.logical_height as f32 - 4.0,
            ),
            &paint(request.focus_rgba, 2.0),
        );
    }
    canvas.restore();
}
