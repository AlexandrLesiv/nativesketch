use sketch_core::Editor;
use std::{
    cell::RefCell,
    collections::BTreeMap,
    panic::{AssertUnwindSafe, catch_unwind},
};
pub(crate) const LIMIT: usize = 64 * 1024 * 1024 + 8;
#[derive(Default)]
pub(crate) struct Registry {
    next: u32,
    pub(crate) editors: BTreeMap<u32, Session>,
    pub(crate) buffers: BTreeMap<u32, Vec<u8>>,
}
pub(crate) struct Session {
    pub(crate) editor: Editor,
    pub(crate) output: u32,
}
thread_local! { pub(crate) static REGISTRY: RefCell<Registry> = RefCell::new(Registry::default()); }
impl Registry {
    pub(crate) fn id(&mut self) -> Option<u32> {
        self.next = self.next.checked_add(1)?;
        Some(self.next)
    }
    pub(crate) fn buffer(&mut self, bytes: Vec<u8>) -> u32 {
        if bytes.len() > LIMIT
            || self.buffers.values().map(Vec::len).sum::<usize>() + bytes.len() > 128 * 1024 * 1024
        {
            return 0;
        }
        let Some(id) = self.id() else { return 0 };
        self.buffers.insert(id, bytes);
        id
    }
}
pub(crate) fn registry<T: Default>(f: impl FnOnce(&mut Registry) -> T) -> T {
    catch_unwind(AssertUnwindSafe(|| {
        REGISTRY.with(|r| f(&mut r.borrow_mut()))
    }))
    .unwrap_or_default()
}
