use crate::*;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProviderId(pub u32);
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProviderResourceId(pub u64);
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResourceAccess {
    pub read: bool,
    pub write: bool,
    pub confirmed_write: bool,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResourceDescriptor {
    pub provider: ProviderId,
    pub resource: ProviderResourceId,
    pub canonical: ProviderResourceId,
    pub access: ResourceAccess,
    pub generation: u64,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResourceRelease {
    pub handle: DestinationHandle,
    pub descriptor: ResourceDescriptor,
}
#[derive(Debug)]
pub struct SessionConfig {
    pub capabilities: Capabilities,
    pub tool: ToolId,
    pub brush: BrushSettings,
}
#[derive(Debug)]
pub enum Intent<'a> {
    Edit(EditorCommand),
    Action(EditorAction),
    Input(SampleBatch<'a>),
    File(FileIntent),
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppError {
    Invalid,
    Stale,
    Busy,
    Capacity,
    CapabilityDenied,
    Unsupported,
    Core(ApplicationError),
    Storage(StorageError),
    Editor(EditorError),
    Scene(sketch_core::SceneError),
}
impl From<ApplicationError> for AppError {
    fn from(v: ApplicationError) -> Self {
        Self::Core(v)
    }
}
impl From<StorageError> for AppError {
    fn from(v: StorageError) -> Self {
        Self::Storage(v)
    }
}
impl From<EditorError> for AppError {
    fn from(v: EditorError) -> Self {
        Self::Editor(v)
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AppStats {
    pub sessions: usize,
    pub operations: usize,
    pub resource_leases: usize,
    pub buffers: usize,
    pub buffer_bytes: usize,
    pub outstanding_saves: usize,
    pub diagnostic_drops: u64,
}
impl AppStats {
    pub fn shutdown_ready(self) -> bool {
        self.sessions == 0
            && self.operations == 0
            && self.resource_leases == 0
            && self.buffers == 0
            && self.outstanding_saves == 0
    }
}
