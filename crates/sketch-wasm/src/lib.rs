//! Dependency-free WebAssembly adapter. Protocol and lifetimes: ../WASM.md.
use sketch_core::{Brush, Editor, Error, Input, Sample};
use std::{
    cell::RefCell,
    collections::BTreeMap,
    panic::{AssertUnwindSafe, catch_unwind},
};
const LIMIT: usize = 64 * 1024 * 1024 + 8;
#[derive(Default)]
struct Registry {
    next: u32,
    editors: BTreeMap<u32, Session>,
    buffers: BTreeMap<u32, Vec<u8>>,
}
struct Session {
    editor: Editor,
    output: u32,
}
thread_local! { static REGISTRY: RefCell<Registry> = RefCell::new(Registry::default()); }
impl Registry {
    fn id(&mut self) -> Option<u32> {
        self.next = self.next.checked_add(1)?;
        Some(self.next)
    }
    fn buffer(&mut self, bytes: Vec<u8>) -> u32 {
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
fn registry<T: Default>(f: impl FnOnce(&mut Registry) -> T) -> T {
    catch_unwind(AssertUnwindSafe(|| {
        REGISTRY.with(|r| f(&mut r.borrow_mut()))
    }))
    .unwrap_or_default()
}
#[unsafe(no_mangle)]
pub extern "C" fn sketch_wasm_protocol_version() -> u32 {
    1
}
#[unsafe(no_mangle)]
pub extern "C" fn sketch_wasm_create() -> u32 {
    registry(|r| {
        let Some(id) = r.id() else { return 0 };
        r.editors.insert(
            id,
            Session {
                editor: Editor::new(),
                output: 0,
            },
        );
        id
    })
}
#[unsafe(no_mangle)]
pub extern "C" fn sketch_wasm_free(handle: u32) -> u32 {
    registry(|r| {
        let Some(s) = r.editors.remove(&handle) else {
            return 1;
        };
        r.buffers.remove(&s.output);
        0
    })
}
#[unsafe(no_mangle)]
pub extern "C" fn sketch_wasm_buffer_new(length: u32) -> u32 {
    registry(|r| {
        if length as usize > LIMIT
            || r.buffers.values().map(Vec::len).sum::<usize>() + length as usize > 128 * 1024 * 1024
        {
            return 0;
        }
        r.buffer(vec![0; length as usize])
    })
}
#[unsafe(no_mangle)]
pub extern "C" fn sketch_wasm_buffer_ptr(token: u32) -> usize {
    registry(|r| r.buffers.get(&token).map_or(0, |b| b.as_ptr() as usize))
}
#[unsafe(no_mangle)]
pub extern "C" fn sketch_wasm_buffer_len(token: u32) -> u32 {
    registry(|r| r.buffers.get(&token).map_or(0, |b| b.len() as u32))
}
#[unsafe(no_mangle)]
pub extern "C" fn sketch_wasm_buffer_free(token: u32) -> u32 {
    registry(|r| {
        if r.editors.values().any(|s| s.output == token) {
            return 2;
        }
        if r.buffers.remove(&token).is_some() {
            0
        } else {
            1
        }
    })
}
#[unsafe(no_mangle)]
pub extern "C" fn sketch_wasm_output(handle: u32) -> u32 {
    registry(|r| r.editors.get(&handle).map_or(0, |s| s.output))
}
fn status(e: Error) -> u32 {
    match e {
        Error::InvalidArgument => 1,
        Error::InvalidState => 2,
        Error::Io => 4,
        Error::Format => 5,
    }
}
struct Reader<'a>(&'a [u8]);
impl Reader<'_> {
    fn take<const N: usize>(&mut self) -> Result<[u8; N], Error> {
        let bytes = self.0.get(..N).ok_or(Error::InvalidArgument)?;
        let value = bytes.try_into().map_err(|_| Error::InvalidArgument)?;
        self.0 = &self.0[N..];
        Ok(value)
    }
    fn u64(&mut self) -> Result<u64, Error> {
        Ok(u64::from_le_bytes(self.take()?))
    }
    fn u32(&mut self) -> Result<u32, Error> {
        Ok(u32::from_le_bytes(self.take()?))
    }
    fn f64(&mut self) -> Result<f64, Error> {
        Ok(f64::from_le_bytes(self.take()?))
    }
    fn done(&self) -> Result<(), Error> {
        if self.0.is_empty() {
            Ok(())
        } else {
            Err(Error::InvalidArgument)
        }
    }
}
fn put64(b: &mut Vec<u8>, x: u64) {
    b.extend_from_slice(&x.to_le_bytes());
}
fn put32(b: &mut Vec<u8>, x: u32) {
    b.extend_from_slice(&x.to_le_bytes());
}
fn doubles(b: &mut Vec<u8>, values: impl IntoIterator<Item = f64>) {
    for x in values {
        put64(b, x.to_bits());
    }
}
fn command(e: &mut Editor, op: u32, bytes: &[u8]) -> Result<Vec<u8>, Error> {
    let mut r = Reader(bytes);
    let mut out = Vec::new();
    // Validate fixed message length before mutations; no trailing garbage accepted.
    let length = match op {
        1 | 5 | 6 | 9 | 10 | 11 | 13 | 14 | 15 | 16 | 17 => Some(0),
        2 => Some(52),
        4 | 19 => Some(8),
        7 | 12 => Some(24),
        8 => Some(4),
        3 | 18 => None,
        _ => return Err(Error::InvalidArgument),
    };
    if length.is_some_and(|n| n != bytes.len()) {
        return Err(Error::InvalidArgument);
    }
    match op {
        1 => {
            let s = e.state();
            put64(&mut out, s.count as u64);
            put64(&mut out, s.selected);
            for n in [
                u32::from(s.can_undo),
                u32::from(s.can_redo),
                u32::from(s.dirty),
                s.interaction,
            ] {
                put32(&mut out, n);
            }
            doubles(
                &mut out,
                [s.bounds.x, s.bounds.y, s.bounds.width, s.bounds.height],
            );
        }
        2 => {
            let pointer = r.u64()?;
            let device = r.u32()?;
            let brush = Brush {
                rgba: [r.f64()?, r.f64()?, r.f64()?, r.f64()?],
                width: r.f64()?,
            };
            e.begin(pointer, device, brush)?;
        }
        3 => {
            let pointer = r.u64()?;
            if r.0.len() % 48 != 0 || r.0.len() / 48 > sketch_core::MAX_SAMPLES {
                return Err(Error::InvalidArgument);
            }
            let mut inputs = Vec::new();
            while !r.0.is_empty() {
                let x = r.f64()?;
                let y = r.f64()?;
                let timestamp = r.f64()?;
                let pressure = r.f64()?;
                let id = r.u64()?;
                let flag = r.u32()?;
                let kind = r.u32()?;
                if flag > 1 || !pressure.is_finite() {
                    return Err(Error::InvalidArgument);
                }
                let s = Sample {
                    x,
                    y,
                    timestamp,
                    id,
                    pressure: if flag == 1 { Some(pressure) } else { None },
                };
                inputs.push(match kind {
                    0 => Input::Actual(s),
                    1 => Input::Predicted(s),
                    2 => Input::Correction(s),
                    _ => return Err(Error::InvalidArgument),
                });
            }
            e.append(pointer, &inputs)?;
        }
        4 => e.commit(r.u64()?)?,
        5 => e.cancel(),
        6 => {
            for c in e.render() {
                put64(&mut out, c.stroke_id);
                put32(&mut out, c.verb);
                put32(&mut out, 0);
                doubles(&mut out, c.rgba);
                doubles(&mut out, c.points);
            }
        }
        7 => e.select_at(r.f64()?, r.f64()?, r.f64()?)?,
        8 => e.select_cycle(r.u32()? as i32)?,
        9 => e.select_clear()?,
        10 => e.delete_selected()?,
        11 => e.resize_begin()?,
        12 => e.resize_update(r.f64()?, r.f64()?, r.f64()?)?,
        13 => e.resize_commit()?,
        14 => e.undo()?,
        15 => e.redo()?,
        16 => e.clear()?,
        17 => {
            let (bytes, revision) = e.export()?;
            put64(&mut out, revision);
            out.extend_from_slice(&bytes);
        }
        18 => {
            e.import(r.0)?;
            r.0 = &[];
        }
        19 => e.mark_saved(r.u64()?)?,
        _ => unreachable!(),
    }
    r.done()?;
    Ok(out)
}
#[unsafe(no_mangle)]
pub extern "C" fn sketch_wasm_command(handle: u32, opcode: u32, input: u32) -> u32 {
    // WASM production panics trap (never unwind); host must discard the instance.
    catch_unwind(AssertUnwindSafe(|| {
        REGISTRY.with(|cell| {
            let mut r = cell.borrow_mut();
            let bytes = if input == 0 {
                Vec::new()
            } else {
                match r.buffers.get(&input) {
                    Some(b) => b.clone(),
                    None => return 1,
                }
            };
            let Some(s) = r.editors.get_mut(&handle) else {
                return 1;
            };
            let old = s.output;
            s.output = 0;
            let result = command(&mut s.editor, opcode, &bytes);
            r.buffers.remove(&old);
            match result {
                Err(e) => status(e),
                Ok(bytes) => {
                    if !bytes.is_empty() {
                        let token = r.buffer(bytes);
                        if token == 0 {
                            return 3;
                        }
                        r.editors.get_mut(&handle).expect("live session").output = token;
                    }
                    0
                }
            }
        })
    }))
    .unwrap_or(6)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn malformed_messages_and_handles_are_errors() {
        assert_eq!(sketch_wasm_command(u32::MAX, 1, 0), 1);
        let h = sketch_wasm_create();
        let b = sketch_wasm_buffer_new(3);
        assert_eq!(sketch_wasm_command(h, 2, b), 1);
        assert_eq!(sketch_wasm_command(h, 1, 999999), 1);
        assert_eq!(sketch_wasm_command(h, 1, 0), 0);
        let out = sketch_wasm_output(h);
        assert_eq!(sketch_wasm_buffer_len(out), 64);
        assert_eq!(sketch_wasm_buffer_free(out), 2);
        assert_eq!(sketch_wasm_free(h), 0);
        assert_eq!(sketch_wasm_buffer_len(out), 0);
        assert_eq!(sketch_wasm_free(h), 1);
        assert_eq!(sketch_wasm_buffer_free(b), 0);
    }
}
