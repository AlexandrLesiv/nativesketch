use super::{Brush, Failure, Text};
#[repr(C)]
#[derive(Clone, Copy)]
pub struct RuntimeConfig {
    pub struct_size: u32,
    pub max_sessions: u32,
    pub max_operations: u32,
    pub max_events: u32,
    pub snapshot_bytes: u64,
    pub build_id: u64,
    pub recovery_root: Text,
    pub diagnostics_root: Text,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SessionConfig {
    pub capabilities: u32,
    pub tool: u32,
    pub brush: Brush,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Location {
    pub grant_id: u64,
    pub path: Text,
    pub provider: u32,
    pub reserved: u32,
}
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct Event {
    pub operation_id: u64,
    pub event_id: u64,
    pub window: u64,
    pub session: u64,
    pub request: u64,
    pub grant_id: u64,
    pub kind: u32,
    pub reserved: u32,
    pub failure: Failure,
}
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct RuntimeState {
    pub pending_operations: u32,
    pub ready_events: u32,
    pub retained_events: u32,
    pub attached_sessions: u32,
    pub needs_poll: u32,
    pub needs_idle: u32,
    pub stopping: u32,
    pub stopped: u32,
    pub wake_after_ms: u64,
    pub dropped_diagnostics: u64,
    pub failed_diagnostic_sinks: u64,
}
