use crate::*;
use sketch_app_wire::{InputBytes, Reply, WireResult, WireWork};
use std::sync::{
    Arc,
    atomic::{AtomicU8, Ordering},
};
use std::thread::ThreadId;
pub(crate) const LIVE: u8 = 0;
pub(crate) const POISONED: u8 = 1;
pub(crate) const STOPPED: u8 = 2;
pub(crate) struct Meta {
    pub id: Handle,
    pub thread: ThreadId,
    pub life: AtomicU8,
}
impl Meta {
    pub fn live(&self) -> Result<()> {
        match self.life.load(Ordering::Acquire) {
            LIVE => Ok(()),
            POISONED => Err(error(10)),
            _ => Err(error(12)),
        }
    }
    pub fn worker(&self) -> Result<()> {
        if self.life.load(Ordering::Acquire) == POISONED {
            Err(error(10))
        } else {
            Ok(())
        }
    }
    pub fn quarantine(&self) {
        let _ = self
            .life
            .compare_exchange(LIVE, POISONED, Ordering::AcqRel, Ordering::Acquire);
    }
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Kind {
    App,
    Work,
    Input,
    Result,
    Buffer,
    Upload,
}
pub(crate) enum Object {
    App,
    Work(WireWork),
    Input(Option<InputBytes>),
    Result(Option<WireResult>),
    Buffer(Reply),
    Upload(Vec<u8>),
}
pub(crate) struct Entry {
    pub id: Handle,
    pub kind: Kind,
    pub meta: Option<Arc<Meta>>,
    pub object: Option<Object>,
}
