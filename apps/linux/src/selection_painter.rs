use gtk::cairo;
use sketch_core::Bounds;
pub(crate) fn paint(cr: &cairo::Context, bounds: Bounds, zoom: f64) -> Result<(), cairo::Error> {
    cr.set_source_rgb(0.05, 0.35, 0.85);
    cr.set_line_width(1.0 / zoom);
    cr.rectangle(bounds.x, bounds.y, bounds.width, bounds.height);
    cr.stroke()?;
    for (x, y) in [
        (bounds.x, bounds.y),
        (bounds.x + bounds.width, bounds.y),
        (bounds.x, bounds.y + bounds.height),
        (bounds.x + bounds.width, bounds.y + bounds.height),
    ] {
        cr.rectangle(x - 4.0 / zoom, y - 4.0 / zoom, 8.0 / zoom, 8.0 / zoom);
        cr.set_source_rgb(1.0, 1.0, 1.0);
        cr.fill_preserve()?;
        cr.set_source_rgb(0.05, 0.35, 0.85);
        cr.stroke()?;
    }
    Ok(())
}
