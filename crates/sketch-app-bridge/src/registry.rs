use crate::{objects::*, *};
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};
const MAX_OBJECTS: usize = 4096;
struct Registry {
    entries: Vec<Entry>,
    next: Handle,
}
fn registry() -> MutexGuard<'static, Registry> {
    static VALUE: OnceLock<Mutex<Registry>> = OnceLock::new();
    VALUE
        .get_or_init(|| {
            Mutex::new(Registry {
                entries: Vec::new(),
                next: 1,
            })
        })
        .lock()
        .unwrap_or_else(|p| p.into_inner())
}
pub(crate) struct Slot {
    pub id: Handle,
    committed: bool,
}
impl Slot {
    pub fn reserve(kind: Kind, meta: Option<Arc<Meta>>) -> Result<Self> {
        let mut r = registry();
        if r.entries.len() == MAX_OBJECTS {
            return Err(error(3));
        }
        if kind == Kind::App && r.entries.iter().filter(|e| e.kind == Kind::App).count() == 16 {
            return Err(error(3));
        }
        let maximum = if cfg!(target_arch = "wasm32") {
            u64::from(u32::MAX)
        } else {
            u64::MAX
        };
        let id = next_identity(&mut r.next, maximum)?;
        r.entries.try_reserve(1).map_err(|_| error(3))?;
        r.entries.push(Entry {
            id,
            kind,
            meta,
            object: None,
        });
        Ok(Self {
            id,
            committed: false,
        })
    }
    pub fn commit(mut self, object: Object, meta: Option<Arc<Meta>>) -> Handle {
        let mut r = registry();
        let e = r
            .entries
            .iter_mut()
            .find(|e| e.id == self.id)
            .expect("reserved slot");
        e.object = Some(object);
        if meta.is_some() {
            e.meta = meta;
        }
        self.committed = true;
        self.id
    }
}
impl Drop for Slot {
    fn drop(&mut self) {
        if !self.committed {
            remove(self.id);
        }
    }
}
pub(crate) fn remove(id: Handle) {
    let mut r = registry();
    if let Some(i) = r.entries.iter().position(|e| e.id == id) {
        r.entries.swap_remove(i);
    }
}
pub(crate) fn meta(id: Handle, kind: Kind) -> Result<Arc<Meta>> {
    let r = registry();
    let e = r.entries.iter().find(|e| e.id == id).ok_or(error(7))?;
    if e.kind != kind {
        return Err(error(1));
    }
    e.meta.clone().ok_or(error(2))
}
pub(crate) fn has_children(id: Handle) -> bool {
    registry()
        .entries
        .iter()
        .any(|e| e.id != id && e.meta.as_ref().is_some_and(|m| m.id == id))
}
pub(crate) struct Checkout {
    pub id: Handle,
    pub object: Option<Object>,
    pub meta: Option<Arc<Meta>>,
}
impl Checkout {
    pub fn take(id: Handle, kind: Kind) -> Result<Self> {
        let mut r = registry();
        let e = r.entries.iter_mut().find(|e| e.id == id).ok_or(error(7))?;
        if e.kind != kind {
            return Err(error(1));
        }
        let object = e.object.take().ok_or(error(9))?;
        Ok(Self {
            id,
            object: Some(object),
            meta: e.meta.clone(),
        })
    }
    pub fn consume(&mut self) {
        self.object = None;
    }
    pub fn owner(&self) -> Arc<Meta> {
        self.meta.as_ref().expect("owned object metadata").clone()
    }
}
impl Drop for Checkout {
    fn drop(&mut self) {
        let mut r = registry();
        if let Some(i) = r.entries.iter().position(|e| e.id == self.id) {
            if self.object.is_some() {
                r.entries[i].object = self.object.take();
            } else {
                r.entries.swap_remove(i);
            }
        }
    }
}
pub(crate) fn guarded<T>(meta: &Meta, f: impl FnOnce() -> Result<T>) -> Result<T> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)).unwrap_or_else(|_| {
        meta.quarantine();
        Err(error(6))
    })
}

fn next_identity(next: &mut Handle, maximum: Handle) -> Result<Handle> {
    if *next >= maximum {
        return Err(error(3));
    }
    let id = *next;
    *next = id + 1;
    Ok(id)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_and_wasm_identity_exhaustion_never_wraps_or_reuses() {
        for maximum in [u64::from(u32::MAX), u64::MAX] {
            let mut next = maximum - 1;
            assert_eq!(next_identity(&mut next, maximum).unwrap(), maximum - 1);
            assert_eq!(next_identity(&mut next, maximum).unwrap_err().status, 3);
            assert_eq!(next_identity(&mut next, maximum).unwrap_err().status, 3);
            assert_eq!(next, maximum);
        }
    }
}
