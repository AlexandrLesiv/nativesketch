use super::Location;
#[repr(C)]
#[derive(Clone, Copy)]
pub struct RecoveryItem {
    pub item_id: u64,
    pub generation: u64,
    pub revision: u64,
    pub created_seconds: u64,
    pub recovery_id: [u8; 16],
    pub display_name_length: u32,
    pub rejected_generations: u32,
    pub display_name: [u8; 256],
}
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct RecoveryInfo {
    pub count: u32,
    pub rejected_sessions: u32,
    pub invalid_markers: u32,
    pub capacity_blocked: u32,
    pub usage_known: u32,
    pub reserved: u32,
    pub used_bytes: u64,
    pub limit_bytes: u64,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Maintenance {
    pub tag: u32,
    pub reserved: u32,
    pub item_id: u64,
    pub window: u64,
    pub location: Location,
}
