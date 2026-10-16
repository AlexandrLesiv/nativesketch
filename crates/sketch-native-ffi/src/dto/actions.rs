/// Additive v1 intent DTO. Unused fields must be zero; no legacy tag is reinterpreted.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct EditorAction {
    pub tag: u32,
    pub option: u32,
    pub factor: f64,
}
