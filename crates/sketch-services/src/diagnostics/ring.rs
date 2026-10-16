use super::{Event, EventCode};
use std::collections::VecDeque;
use std::io;
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DiagnosticStats {
    pub dropped: u64,
    pub rate_limited: u64,
    pub sink_failures: u64,
}
pub struct DiagnosticRing {
    events: VecDeque<Event>,
    capacity: usize,
    per_second: u32,
    window_ms: u64,
    repetitions: [u32; EventCode::COUNTERS],
    stats: DiagnosticStats,
}
impl DiagnosticRing {
    pub fn new(capacity: usize, per_second: u32) -> io::Result<Self> {
        if capacity == 0 || capacity > 4096 || per_second == 0 {
            return Err(io::ErrorKind::InvalidInput.into());
        }
        Ok(Self {
            events: VecDeque::with_capacity(capacity),
            capacity,
            per_second,
            window_ms: 0,
            repetitions: [0; EventCode::COUNTERS],
            stats: DiagnosticStats::default(),
        })
    }
    /// Bounded owner-side bookkeeping; caller supplies a monotonic millisecond clock.
    pub fn record(&mut self, event: Event, now_ms: u64) {
        if now_ms.saturating_sub(self.window_ms) >= 1000 {
            self.window_ms = now_ms;
            self.repetitions = [0; EventCode::COUNTERS];
        }
        let count = &mut self.repetitions[event.code.counter()];
        if *count >= self.per_second {
            self.stats.rate_limited = self.stats.rate_limited.saturating_add(1);
            return;
        }
        *count += 1;
        if self.events.len() == self.capacity {
            self.events.pop_front();
            self.stats.dropped = self.stats.dropped.saturating_add(1);
        }
        self.events.push_back(event);
    }
    pub fn stats(&self) -> DiagnosticStats {
        self.stats
    }
    pub fn len(&self) -> usize {
        self.events.len()
    }
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
    /// Call off the UI/ink path. Failed records remain bounded for a later explicit retry.
    pub fn drain_to(&mut self, mut sink: impl FnMut(Event) -> io::Result<()>) {
        while let Some(event) = self.events.front().copied() {
            let delivered = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| sink(event)));
            if !matches!(delivered, Ok(Ok(()))) {
                self.stats.sink_failures = self.stats.sink_failures.saturating_add(1);
                break;
            }
            self.events.pop_front();
        }
    }
}
