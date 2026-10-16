use crate::*;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkKind {
    Write,
    Read,
    PrepareNew,
    Retire,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorkInfo {
    pub operation: OperationId,
    pub session: SessionHandle,
    pub request: RequestId,
    pub kind: WorkKind,
    pub resource: Option<ResourceDescriptor>,
}
#[derive(Debug, Clone, Copy)]
pub(crate) struct Stamp {
    pub app: u64,
    pub info: WorkInfo,
}
#[derive(Debug)]
pub enum WorkItem {
    Write(ProviderWrite),
    Read(ProviderRead),
    PrepareNew(ProviderNew),
    Retire(RetiredState),
}
impl WorkItem {
    pub fn info(&self) -> WorkInfo {
        match self {
            Self::Write(w) => w.stamp.info,
            Self::Read(w) => w.stamp.info,
            Self::PrepareNew(w) => w.stamp.info,
            Self::Retire(w) => w.stamp.unwrap().info,
        }
    }
}
#[derive(Debug)]
pub(crate) enum Payload {
    Write(WriteReceipt),
    Save(Option<SaveCompletion>),
    Candidate(Box<PreparedReplacement>),
    Failure(Option<ReplacementFailure>),
    Retired,
}
#[derive(Debug)]
pub struct WorkResult {
    pub(crate) stamp: Box<Stamp>,
    pub(crate) payload: Payload,
}
impl WorkResult {
    pub fn info(&self) -> WorkInfo {
        self.stamp.info
    }
}
#[derive(Debug)]
pub(crate) enum Retired {
    Editor(EditorSession),
    App(ApplicationSession),
    Candidate(Box<PreparedReplacement>),
}
#[derive(Debug)]
pub struct RetiredState {
    pub(crate) stamp: Option<Stamp>,
    pub(crate) states: Vec<Retired>,
}
impl RetiredState {
    /// Explicit worker call: release potentially large document/history state off input.
    pub fn dispose(self) -> WorkResult {
        let stamp = self.stamp.expect("retirement issued by app");
        for state in self.states {
            match state {
                Retired::Editor(v) => drop(v),
                Retired::App(v) => drop(v),
                Retired::Candidate(v) => drop(v),
            }
        }
        WorkResult {
            stamp: Box::new(stamp),
            payload: Payload::Retired,
        }
    }
}
