/// Layout is mirrored by SketchRenderStatsV1. Durations are CPU stages, not pen latency.
#[derive(Debug, Default, Clone, Copy)]
#[repr(C)]
pub struct RenderStats {
    pub context_generation: u64,
    pub scene_prepare_ns: u64,
    pub render_submit_ns: u64,
    pub geometry_bytes: u64,
    pub path_bytes: u64,
    pub background_bytes: u64,
    pub path_hits: u64,
    pub path_misses: u64,
    pub background_hit: u32,
    pub reserved: u32,
}
