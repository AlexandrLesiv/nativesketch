use super::Brush;
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Selection {
    pub x: f64,
    pub y: f64,
    pub tolerance: f64,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Stroke {
    pub pointer: u64,
    pub device: u32,
    pub reserved: u32,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Transform {
    pub scale: f64,
    pub anchor_x: f64,
    pub anchor_y: f64,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub union CommandPayload {
    pub tool: u32,
    pub brush: Brush,
    pub selection: Selection,
    pub direction: i32,
    pub stroke: Stroke,
    pub transform: Transform,
    pub recovery_item: u64,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Command {
    pub tag: u32,
    pub reserved: u32,
    pub payload: CommandPayload,
}
