use super::{RuntimeConfig, io_state::IoState, work::*};
use crate::storage::NoFaults;
use std::{
    panic::{AssertUnwindSafe, catch_unwind},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{Receiver, RecvTimeoutError, SyncSender},
    },
    time::Duration,
};
pub(super) fn run(
    config: RuntimeConfig,
    input: Receiver<Box<WorkerRequest>>,
    output: SyncSender<WorkerResult>,
    stop: &AtomicBool,
) {
    let mut state = IoState::new(config);
    let started = WorkerValue::Started {
        recovery: state.recovery.as_ref().map(|_| ()).map_err(|error| *error),
        diagnostics: state
            .diagnostics
            .as_ref()
            .map(|_| ())
            .map_err(|error| *error),
    };
    if output.send(state.result(0, started)).is_err() {
        return;
    }
    loop {
        let request = match input.recv_timeout(Duration::from_millis(25)) {
            Ok(request) => request,
            Err(RecvTimeoutError::Disconnected) => break,
            Err(RecvTimeoutError::Timeout) if stop.load(Ordering::Acquire) => break,
            Err(RecvTimeoutError::Timeout) => continue,
        };
        let WorkerRequest { operation, work } = *request;
        let value = catch_unwind(AssertUnwindSafe(|| state.perform(work, &NoFaults)))
            .unwrap_or(WorkerValue::Failure(NativeFailure::Panic));
        if output.send(state.result(operation, value)).is_err() {
            break;
        }
    }
}
