use gtk::prelude::DrawingAreaExtManual;

use crate::presenter::Ui;

pub(crate) fn connect(ui: &Ui) {
    let model = ui.model.clone();
    let canvas = &ui.canvas;
    {
        let m = model.clone();
        canvas.set_draw_func(move |_, cr, w, h| {
            let mut m = m.borrow_mut();
            m.layout(w, h);
            cr.set_source_rgb(1.0, 1.0, 1.0);
            let _ = cr.paint();
            cr.translate(m.offset.0, m.offset.1);
            cr.scale(m.zoom, m.zoom);
            let mut id = None;
            for command in m.editor.render() {
                if id != Some(command.stroke_id) {
                    if id.is_some() {
                        let _ = cr.fill();
                    }
                    cr.new_path();
                    id = Some(command.stroke_id);
                    let c = command.rgba;
                    cr.set_source_rgba(c[0], c[1], c[2], c[3]);
                }
                let p = command.points;
                match command.verb {
                    0 => cr.move_to(p[0], p[1]),
                    1 => cr.line_to(p[0], p[1]),
                    2 => cr.curve_to(p[0], p[1], p[2], p[3], p[4], p[5]),
                    3 => cr.close_path(),
                    _ => {}
                }
            }
            let _ = cr.fill();
            let s = m.editor.state();
            if s.selected != 0 {
                let b = s.bounds;
                cr.set_source_rgb(0.05, 0.35, 0.85);
                cr.set_line_width(1.0 / m.zoom);
                cr.rectangle(b.x, b.y, b.width, b.height);
                let _ = cr.stroke();
                for (x, y) in [
                    (b.x, b.y),
                    (b.x + b.width, b.y),
                    (b.x, b.y + b.height),
                    (b.x + b.width, b.y + b.height),
                ] {
                    cr.rectangle(
                        x - 4.0 / m.zoom,
                        y - 4.0 / m.zoom,
                        8.0 / m.zoom,
                        8.0 / m.zoom,
                    );
                    cr.set_source_rgb(1.0, 1.0, 1.0);
                    let _ = cr.fill_preserve();
                    cr.set_source_rgb(0.05, 0.35, 0.85);
                    let _ = cr.stroke();
                }
            }
        });
    }
}
