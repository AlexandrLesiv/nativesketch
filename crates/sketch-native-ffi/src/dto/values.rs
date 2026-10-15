#[repr(C)]
#[derive(Clone, Copy)]
pub struct Text {
    pub bytes: *const u8,
    pub length: u32,
}
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct Brush {
    pub red: f64,
    pub green: f64,
    pub blue: f64,
    pub alpha: f64,
    pub width: f64,
}
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct Bounds {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct Context {
    pub session: u64,
    pub request: u64,
    pub interaction: u64,
}
#[repr(C)]
#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub struct EffectToken {
    pub owner: u64,
    pub session: u64,
    pub serial: u64,
    pub request: u64,
}
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct Sample {
    pub x: f64,
    pub y: f64,
    pub timestamp: f64,
    pub pressure: f64,
    pub sample_id: u64,
    pub has_pressure: u32,
    pub kind: u32,
}
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct Path {
    pub stroke_id: u64,
    pub verb: u32,
    pub reserved: u32,
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
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct SceneInfo {
    pub session: u64,
    pub revision: u64,
    pub generation: u64,
    pub command_count: u64,
}
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct Failure {
    pub domain: u32,
    pub code: u32,
    pub operation: u32,
    pub reserved: u32,
    pub session: u64,
    pub request: u64,
    pub operation_id: u64,
}
