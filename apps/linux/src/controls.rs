use adw::prelude::*;
use gtk::gdk;

#[derive(Clone)]
pub(crate) struct Controls {
    pub(crate) new: gtk::Button,
    pub(crate) open: gtk::Button,
    pub(crate) save: gtk::Button,
    pub(crate) undo: gtk::Button,
    pub(crate) redo: gtk::Button,
    pub(crate) pen: gtk::ToggleButton,
    pub(crate) select: gtk::ToggleButton,
    pub(crate) color: gtk::ColorDialogButton,
    pub(crate) width: gtk::SpinButton,
    pub(crate) previous: gtk::Button,
    pub(crate) next: gtk::Button,
    pub(crate) scale: gtk::SpinButton,
    pub(crate) apply: gtk::Button,
    pub(crate) delete: gtk::Button,
    pub(crate) out: gtk::Button,
    pub(crate) reset: gtk::Button,
    pub(crate) zoom_in: gtk::Button,
}
impl Controls {
    pub(crate) fn new(header: &adw::HeaderBar, root: &gtk::Box) -> Self {
        let new = button("New");
        let open = button("Open…");
        let save = button("Save…");
        let undo = button("Undo");
        let redo = button("Redo");
        header.pack_start(&new);
        header.pack_start(&open);
        header.pack_start(&save);
        header.pack_end(&redo);
        header.pack_end(&undo);
        let bar = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        bar.set_margin_start(12);
        bar.set_margin_end(12);
        bar.set_margin_top(8);
        bar.set_margin_bottom(8);
        let scroll = gtk::ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::Automatic)
            .vscrollbar_policy(gtk::PolicyType::Never)
            .child(&bar)
            .build();
        root.append(&scroll);
        let pen = gtk::ToggleButton::with_label("Pen");
        pen.set_active(true);
        let select = gtk::ToggleButton::with_label("Select");
        select.set_group(Some(&pen));
        bar.append(&pen);
        bar.append(&select);
        let color = gtk::ColorDialogButton::new(Some(gtk::ColorDialog::new()));
        color.set_rgba(&gdk::RGBA::new(0.14, 0.39, 0.92, 1.0));
        color.set_tooltip_text(Some("Pen color"));
        color.update_property(&[gtk::accessible::Property::Label("Pen color")]);
        bar.append(&color);
        bar.append(&gtk::Label::new(Some("Width")));
        let width = gtk::SpinButton::with_range(0.1, 100.0, 0.5);
        width.set_value(4.0);
        width.update_property(&[gtk::accessible::Property::Label("Pen width")]);
        bar.append(&width);
        let previous = button("Previous stroke");
        let next = button("Next stroke");
        bar.append(&previous);
        bar.append(&next);
        bar.append(&gtk::Label::new(Some("Scale %")));
        let scale = gtk::SpinButton::with_range(1.0, 10000.0, 10.0);
        scale.set_value(100.0);
        scale.update_property(&[gtk::accessible::Property::Label("Selection scale percent")]);
        bar.append(&scale);
        let apply = button("Apply");
        bar.append(&apply);
        let delete = button("Delete");
        bar.append(&delete);
        let out = button("−");
        out.set_tooltip_text(Some("Zoom out"));
        let reset = button("100%");
        let zoom_in = button("+");
        zoom_in.set_tooltip_text(Some("Zoom in"));
        bar.append(&out);
        bar.append(&reset);
        bar.append(&zoom_in);

        Self {
            new,
            open,
            save,
            undo,
            redo,
            pen,
            select,
            color,
            width,
            previous,
            next,
            scale,
            apply,
            delete,
            out,
            reset,
            zoom_in,
        }
    }
}
fn button(label: &str) -> gtk::Button {
    gtk::Button::with_label(label)
}
