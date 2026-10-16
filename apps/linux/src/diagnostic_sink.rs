use crate::{
    diagnostic_budget::{Budget, Level},
    diagnostics::DROPPED,
};
use std::{
    io::Write,
    sync::{atomic::Ordering, mpsc::Receiver},
    time::{Duration, Instant},
};
pub(crate) struct SinkState {
    budget: Budget,
    reported: [u64; 5],
    sink_failures: u64,
    last_summary: Duration,
}
impl SinkState {
    pub fn new() -> Self {
        Self {
            budget: Budget::new(),
            reported: [0; 5],
            sink_failures: 0,
            last_summary: Duration::ZERO,
        }
    }
    pub fn tick(
        &mut self,
        channels: &[Receiver<String>; 2],
        now: Duration,
        drops: [u64; 2],
        sink: &mut impl Write,
    ) {
        for (index, level) in [(0, Level::Failure), (1, Level::Event)] {
            for message in channels[index].try_iter().take(32) {
                if self.budget.admit(now, level) && !emit(sink, &message) {
                    self.sink_failures = self.sink_failures.saturating_add(1);
                }
            }
        }
        if now.saturating_sub(self.last_summary) >= Duration::from_secs(1) {
            self.last_summary = now;
            let suppressed = self.budget.suppressed();
            let totals = [
                drops[0],
                drops[1],
                suppressed[0],
                suppressed[1],
                self.sink_failures,
            ];
            if totals != self.reported {
                let summary = format!(
                    "event=diagnostics_suppression failure_queue_drops={} event_queue_drops={} failure_rate_drops={} event_rate_drops={} sink_failures={}",
                    totals[0], totals[1], totals[2], totals[3], totals[4]
                );
                if emit(sink, &summary) {
                    self.reported = totals;
                } else {
                    self.sink_failures = self.sink_failures.saturating_add(1);
                }
            }
        }
    }
}
/// Only this dedicated thread touches the sink. Slow, broken or panicking sinks cannot
/// hold an editor mutation owner or prevent a provider from returning its actual receipt.
pub(crate) fn run(channels: [Receiver<String>; 2]) {
    let start = Instant::now();
    let mut state = SinkState::new();
    loop {
        state.tick(
            &channels,
            start.elapsed(),
            [
                DROPPED[0].load(Ordering::Relaxed),
                DROPPED[1].load(Ordering::Relaxed),
            ],
            &mut std::io::stderr(),
        );
        std::thread::sleep(Duration::from_millis(25));
    }
}
fn emit(sink: &mut impl Write, message: &str) -> bool {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        sink.write_all(message.as_bytes())
            .and_then(|()| sink.write_all(b"\n"))
            .is_ok()
    }))
    .unwrap_or(false)
}
