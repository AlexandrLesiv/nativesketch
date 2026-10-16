use crate::diagnostic_budget::Level;
use std::{
    fmt::{Arguments, Write},
    sync::{
        OnceLock,
        atomic::{AtomicU64, Ordering},
        mpsc::{self, SyncSender},
    },
};
static LOGGER: OnceLock<[SyncSender<String>; 2]> = OnceLock::new();
pub(crate) static DROPPED: [AtomicU64; 2] = [AtomicU64::new(0), AtomicU64::new(0)];
const MAX_RECORD: usize = 1024;
/// Startup only: never initializes or performs sink I/O from an input/provider callback.
pub(crate) fn start() {
    let (failures, receive_failures) = mpsc::sync_channel(32);
    let (events, receive_events) = mpsc::sync_channel(32);
    if LOGGER.set([failures, events]).is_err() {
        return;
    }
    // Failure to spawn only disconnects the bounded channels: record() remains fallible-free.
    let _worker = std::thread::Builder::new()
        .name("sketch-diagnostics".into())
        .spawn(move || crate::diagnostic_sink::run([receive_failures, receive_events]));
}
pub(crate) fn record(message: Arguments<'_>) {
    enqueue(Level::Failure, message);
}
pub(crate) fn event(message: Arguments<'_>) {
    enqueue(Level::Event, message);
}
fn enqueue(level: Level, message: Arguments<'_>) {
    let index = level as usize;
    let Some(channels) = LOGGER.get() else {
        DROPPED[index].fetch_add(1, Ordering::Relaxed);
        return;
    };
    if !send(channels, level, message) {
        DROPPED[index].fetch_add(1, Ordering::Relaxed);
    }
}
pub(crate) fn send(
    channels: &[SyncSender<String>; 2],
    level: Level,
    message: Arguments<'_>,
) -> bool {
    let mut bounded = Bounded(String::with_capacity(MAX_RECORD));
    let _format = bounded.write_fmt(message);
    channels[level as usize].try_send(bounded.0).is_ok()
}

struct Bounded(String);
impl Write for Bounded {
    fn write_str(&mut self, text: &str) -> std::fmt::Result {
        let mut end = text.len().min(MAX_RECORD - self.0.len());
        while !text.is_char_boundary(end) {
            end -= 1;
        }
        self.0.push_str(&text[..end]);
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn records_remain_bounded_at_utf8_boundaries() {
        let mut record = Bounded(String::new());
        write!(&mut record, "{}", "🖊".repeat(1000)).unwrap();
        assert_eq!(record.0.len(), MAX_RECORD);
        record.write_str("more").unwrap();
        assert_eq!(record.0.len(), MAX_RECORD);
    }
}
