use crate::{convert, dto};
use sketch_services::native_runtime::NativeRuntime;
use std::{
    cell::Cell,
    panic::{AssertUnwindSafe, catch_unwind},
    rc::Rc,
    thread::{self, ThreadId},
};
pub struct Runtime {
    pub core: NativeRuntime,
    pub owner: ThreadId,
    pub poisoned: bool,
}
pub struct Session {
    pub runtime: *mut Runtime,
    pub window: u64,
    pub scene_active: Rc<Cell<bool>>,
}
pub struct Scene {
    pub owner: ThreadId,
    pub active: Rc<Cell<bool>>,
    pub value: sketch_core::application::SceneSnapshot,
}
pub fn entry(run: impl FnOnce() -> Result<(), u32>) -> u32 {
    match catch_unwind(AssertUnwindSafe(run)) {
        Ok(Ok(())) => 0,
        Ok(Err(status)) => status,
        Err(_) => 6,
    }
}
pub unsafe fn runtime(
    pointer: *mut Runtime,
    allow_poison: bool,
    run: impl FnOnce(&mut Runtime) -> Result<(), u32>,
) -> u32 {
    if pointer.is_null() {
        return 1;
    }
    // Serial uniquely owned handle is a C contract; check affinity before mutable access.
    if unsafe { (*pointer).owner } != thread::current().id() {
        return 11;
    }
    let runtime = unsafe { &mut *pointer };
    if runtime.poisoned && !allow_poison {
        return 10;
    }
    match catch_unwind(AssertUnwindSafe(|| run(runtime))) {
        Ok(Ok(())) => 0,
        Ok(Err(status)) => status,
        Err(_) => {
            runtime.poisoned = true;
            6
        }
    }
}
pub unsafe fn session<'a>(
    pointer: *mut Session,
    runtime: *mut Runtime,
) -> Result<&'a Session, u32> {
    if pointer.is_null() {
        return Err(1);
    }
    let session = unsafe { &*pointer };
    if session.runtime != runtime {
        return Err(11);
    }
    Ok(session)
}
pub unsafe fn read<T: Copy>(pointer: *const T) -> Result<T, u32> {
    if pointer.is_null() {
        Err(1)
    } else {
        Ok(unsafe { pointer.read() })
    }
}
pub unsafe fn slice<'a, T>(
    pointer: *const T,
    count: usize,
    maximum: usize,
) -> Result<&'a [T], u32> {
    if count > maximum || (count > 0 && pointer.is_null()) {
        return Err(1);
    }
    if count == 0 {
        Ok(&[])
    } else {
        Ok(unsafe { std::slice::from_raw_parts(pointer, count) })
    }
}
pub unsafe fn text(value: dto::Text, maximum: usize) -> Result<String, u32> {
    let bytes = unsafe { slice(value.bytes, value.length as usize, maximum)? };
    if bytes.contains(&0) {
        return Err(1);
    }
    std::str::from_utf8(bytes).map(str::to_owned).map_err(|_| 1)
}
pub fn require_output<T>(pointer: *mut T) -> Result<(), u32> {
    if pointer.is_null() { Err(1) } else { Ok(()) }
}
pub fn status<T>(
    value: Result<T, sketch_services::native_runtime::RuntimeError>,
) -> Result<T, u32> {
    value.map_err(convert::failures::status)
}
