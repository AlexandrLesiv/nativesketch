use crate::{diagnostic_budget::Level, diagnostic_sink::SinkState, diagnostics::send};
use std::{io::Write, sync::mpsc, time::Duration};
struct Broken;
impl Write for Broken {
    fn write(&mut self, _: &[u8]) -> std::io::Result<usize> {
        Err(std::io::ErrorKind::BrokenPipe.into())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
struct Panics;
impl Write for Panics {
    fn write(&mut self, _: &[u8]) -> std::io::Result<usize> {
        panic!("test sink failure")
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
#[test]
fn full_event_queue_preserves_failure_capacity_and_reports_real_totals_after_recovery() {
    let (failures, rx_failures) = mpsc::sync_channel(32);
    let (events, rx_events) = mpsc::sync_channel(32);
    let tx = [failures, events];
    let rx = [rx_failures, rx_events];
    let mut state = SinkState::new();
    for _ in 0..32 {
        assert!(send(
            &tx,
            Level::Event,
            format_args!("event=save_completed")
        ));
    }
    assert!(!send(&tx, Level::Event, format_args!("event=overflow")));
    assert!(send(
        &tx,
        Level::Failure,
        format_args!("event=write_failed")
    ));
    state.tick(&rx, Duration::ZERO, [0, 1], &mut Broken);
    let mut recovered = Vec::new();
    state.tick(&rx, Duration::from_secs(1), [0, 1], &mut recovered);
    let output = String::from_utf8(recovered).unwrap();
    assert!(output.contains("event_queue_drops=1"));
    assert!(output.contains("event_rate_drops=16"));
    assert!(output.contains("sink_failures=17"));
    assert!(send(
        &tx,
        Level::Failure,
        format_args!("event=later_failure")
    ));
    let mut recovered = Vec::new();
    state.tick(&rx, Duration::from_secs(2), [0, 1], &mut recovered);
    assert!(
        String::from_utf8(recovered)
            .unwrap()
            .contains("event=later_failure")
    );
}
#[test]
fn panicking_sink_is_contained_and_recovery_reports_failure() {
    let (a, ra) = mpsc::sync_channel(1);
    let (_b, rb) = mpsc::sync_channel(1);
    a.send("event=failed".into()).unwrap();
    let mut state = SinkState::new();
    let channels = [ra, rb];
    state.tick(&channels, Duration::ZERO, [0, 0], &mut Panics);
    let mut recovered = Vec::new();
    state.tick(&channels, Duration::from_secs(1), [0, 0], &mut recovered);
    assert!(
        String::from_utf8(recovered)
            .unwrap()
            .contains("sink_failures=1")
    );
}
