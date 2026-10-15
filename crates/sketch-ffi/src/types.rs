#[repr(C)]
#[derive(Clone, Copy)]
pub struct SketchSample {
    pub x: f64,
    pub y: f64,
    pub timestamp: f64,
    pub pressure: f64,
    pub sample_id: u64,
    pub has_pressure: u32,
    pub kind: u32,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SketchBrush {
    pub red: f64,
    pub green: f64,
    pub blue: f64,
    pub alpha: f64,
    pub width: f64,
}
#[repr(C)]
#[derive(Default, Clone, Copy)]
pub struct SketchBounds {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}
#[repr(C)]
#[derive(Default, Clone, Copy)]
pub struct SketchState {
    pub stroke_count: u64,
    pub selected_id: u64,
    pub can_undo: u32,
    pub can_redo: u32,
    pub dirty: u32,
    pub interaction: u32,
    pub selection: SketchBounds,
}
#[repr(C)]
#[derive(Default, Clone, Copy)]
pub struct SketchPathCommand {
    pub stroke_id: u64,
    pub verb: u32,
    pub red: f64,
    pub green: f64,
    pub blue: f64,
    pub alpha: f64,
    pub x1: f64,
    pub y1: f64,
    pub x2: f64,
    pub y2: f64,
    pub x3: f64,
    pub y3: f64,
}
/// Borrowed UTF-8 bytes, never retained by Rust.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SketchSourceText {
    pub bytes: *const u8,
    pub length: usize,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SketchSourceVersion {
    pub namespace_id: SketchSourceText,
    pub document_id: SketchSourceText,
    pub revision_id: SketchSourceText,
    pub parent_revision_id: SketchSourceText,
    pub content_sha256: SketchSourceText,
    pub server_generation: u64,
    pub kind: u32,
}
