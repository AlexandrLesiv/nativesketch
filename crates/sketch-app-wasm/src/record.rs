use crate::bridge;
use std::sync::Mutex;
static RECORD: Mutex<[u8; 128]> = Mutex::new([0; 128]);
#[derive(Default)]
pub(crate) struct Output {
    pub flags: u32,
    pub primary: u64,
    pub reply: u64,
    pub work: u64,
    pub result: u64,
    pub count: usize,
    pub input: u64,
}
impl Output {
    pub fn primary(value: u64) -> Self {
        Self {
            primary: value,
            ..Self::default()
        }
    }
    pub fn reply(value: u64) -> Self {
        Self {
            reply: value,
            ..Self::default()
        }
    }
    pub fn consumed(flags: u32) -> Self {
        Self {
            flags,
            ..Self::default()
        }
    }
}
pub(crate) fn run(f: impl FnOnce() -> bridge::Result<Output>) -> u32 {
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)).unwrap_or_else(|_| {
        Err(bridge::WireError {
            status: 6,
            failure: bridge::Failure::default(),
        })
    });
    let mut record = RECORD.lock().unwrap_or_else(|p| p.into_inner());
    record.fill(0);
    match result {
        Ok(o) => {
            let values = [
                u64::from(o.flags),
                o.primary,
                o.reply,
                o.work,
                o.result,
                o.count as u64,
                o.input,
            ];
            for (i, value) in values.into_iter().enumerate() {
                // The shared registry limits Wasm IDs to u32 and buffers to <2^32.
                record[4 + i * 4..8 + i * 4].copy_from_slice(
                    &u32::try_from(value)
                        .expect("Wasm bounded output")
                        .to_le_bytes(),
                );
            }
            0
        }
        Err(e) => {
            record[..4].copy_from_slice(&e.status.to_le_bytes());
            record[32..104].copy_from_slice(&e.packet(0, 0));
            e.status
        }
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn ns_v2_wasm_result_ptr() -> u32 {
    let r = RECORD.lock().unwrap_or_else(|p| p.into_inner());
    u32::try_from(r.as_ptr() as usize).unwrap_or(0)
}
