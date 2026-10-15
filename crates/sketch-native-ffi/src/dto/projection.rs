use super::{Brush, EffectToken, Failure};
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct Projection {
    pub window: u64,
    pub session: u64,
    pub revision: u64,
    pub scene_generation: u64,
    pub interaction: u64,
    pub stroke_count: u64,
    pub selected_id: u64,
    pub destination: u64,
    pub binding_generation: u64,
    pub binding_grant: u64,
    pub continuation: EffectToken,
    pub brush: Brush,
    pub tool: u32,
    pub interaction_kind: u32,
    pub capabilities: u32,
    pub dirty: u32,
    pub can_undo: u32,
    pub can_redo: u32,
    pub file_phase: u32,
    pub navigation: u32,
    pub outstanding_saves: u32,
    pub preparing_operations: u32,
    pub recovery_state: u32,
    pub reserved: u32,
}
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct Availability {
    pub enabled: u32,
    pub reason: u32,
}
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct Outcome {
    pub projection: Projection,
    pub effect: EffectToken,
    pub failure: Failure,
    pub operation_id: u64,
    pub captured_session: u64,
    pub captured_revision: u64,
    pub written_destination: u64,
    pub written_revision: u64,
    pub effect_kind: u32,
    pub change_flags: u32,
    pub invalidation: u32,
    pub save_purpose: u32,
    pub has_save_result: u32,
    pub save_result: u32,
    pub acknowledgement: u32,
    pub durability: u32,
    pub retry_action: u32,
    pub reserved: u32,
}
