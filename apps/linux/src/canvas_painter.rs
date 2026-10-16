use gtk::prelude::DrawingAreaExtManual;

use crate::presenter::Ui;

pub(crate) fn connect(ui: &Ui) {
    let model = ui.model.clone();
    let canvas = &ui.canvas;
    let closing = ui.closing.clone();
    {
        let m = model.clone();
        let cache = std::cell::RefCell::new(sketch_core::SceneCache::default());
        canvas.set_draw_func(move |_, cr, w, h| {
            if closing.get() {
                return;
            }
            let mut cache = cache.borrow_mut();
            let mut m = m.borrow_mut();
            m.layout(w, h);
            cr.set_source_rgb(1.0, 1.0, 1.0);
            if let Err(error) = cr.paint() {
                crate::diagnostics::record(format_args!(
                    "event=canvas_clear_failed error={error:?}"
                ));
                return;
            }
            cr.translate(m.offset.0, m.offset.1);
            cr.scale(m.zoom, m.zoom);
            let viewport = sketch_core::RenderViewport {
                x: -m.offset.0 / m.zoom,
                y: -m.offset.1 / m.zoom,
                width: w as f64 / m.zoom,
                height: h as f64 / m.zoom,
                margin: 2.0 / m.zoom,
            };
            match m.owner.app.scene(m.owner.session, &mut cache, viewport) {
                Ok(scene) => {
                    if let Err(error) = crate::scene_painter::paint(cr, &scene) {
                        crate::diagnostics::record(format_args!(
                            "event=canvas_failure error={error:?}"
                        ));
                    }
                }
                Err(error) => {
                    crate::diagnostics::record(format_args!("event=scene_failure error={error:?}"))
                }
            }
            let s = m.owner.state();
            if s.selected != 0 {
                let b = m
                    .owner
                    .app
                    .selected_bounds(m.owner.session)
                    .unwrap_or_default();
                if let Err(error) = crate::selection_painter::paint(cr, b, m.zoom) {
                    crate::diagnostics::record(format_args!(
                        "event=selection_paint_failed error={error:?}"
                    ));
                }
            }
        });
    }
}
