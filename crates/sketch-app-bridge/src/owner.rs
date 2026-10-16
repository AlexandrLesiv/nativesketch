use crate::{objects::*, registry::*, *};
use sketch_app_wire::{TransportLimits, WireApp};
use std::{
    cell::RefCell,
    sync::{
        Arc,
        atomic::{AtomicU8, Ordering},
    },
};
struct LocalApp {
    meta: Arc<Meta>,
    app: WireApp,
}
impl Drop for LocalApp {
    fn drop(&mut self) {
        self.meta.life.store(STOPPED, Ordering::Release);
        remove(self.meta.id);
    }
}
thread_local! { static APPS: RefCell<Vec<LocalApp>> = const { RefCell::new(Vec::new()) }; }
pub fn app_new(packet: &[u8]) -> Result<Handle> {
    control(packet)?;
    let slot = Slot::reserve(Kind::App, None)?;
    let meta = Arc::new(Meta {
        id: slot.id,
        thread: std::thread::current().id(),
        life: AtomicU8::new(LIVE),
    });
    APPS.try_with(|cell| {
        let mut apps = cell.try_borrow_mut().map_err(|_| error(9))?;
        apps.try_reserve(1).map_err(|_| error(3))?;
        let app = guarded(&meta, || {
            WireApp::bootstrap(packet, TransportLimits::default())
        })?;
        apps.push(LocalApp {
            meta: meta.clone(),
            app,
        });
        Ok(slot.commit(Object::App, Some(meta)))
    })
    .map_err(|_| error(12))?
}
pub(crate) fn with_app<T>(
    id: Handle,
    f: impl FnOnce(&mut WireApp, Arc<Meta>) -> Result<T>,
) -> Result<T> {
    let meta = meta(id, Kind::App)?;
    if meta.thread != std::thread::current().id() {
        return Err(error(11));
    }
    meta.live()?;
    APPS.try_with(|cell| {
        let mut apps = cell.try_borrow_mut().map_err(|_| error(9))?;
        let local = apps.iter_mut().find(|a| a.meta.id == id).ok_or(error(7))?;
        guarded(&meta, || f(&mut local.app, meta.clone()))
    })
    .map_err(|_| error(12))?
}
pub fn app_call(id: Handle, packet: &[u8]) -> Result<Handle> {
    control(packet)?;
    with_app(id, |app, meta| {
        let slot = Slot::reserve(Kind::Buffer, Some(meta))?;
        let reply = app.call(packet)?;
        Ok(slot.commit(Object::Buffer(reply), None))
    })
}
pub fn app_free(id: Handle) -> Result<()> {
    with_app(id, |app, _| {
        if has_children(id) || !app.shutdown_ready() {
            return Err(error(9));
        }
        Ok(())
    })?;
    APPS.with(|cell| {
        let mut apps = cell.borrow_mut();
        let i = apps
            .iter()
            .position(|a| a.meta.id == id)
            .expect("checked owner");
        apps.swap_remove(i);
    });
    Ok(())
}
