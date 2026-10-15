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
    controls_stop: &AtomicBool,
) {
    let mut state = IoState::new(config);
    #[cfg(feature = "native-test-controls")]
    let controls = super::test_control::TestControls::default();
    #[cfg(not(feature = "native-test-controls"))]
    let _ = controls_stop;
    state.root_events();
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
        let WorkerRequest {
            operation,
            correlation,
            work,
        } = *request;
        let (subsystem, kind) = super::io_events::classify(&work);
        let started = std::time::Instant::now();
        state.operation_event(operation, correlation, subsystem, kind, None, 0);
        let value = catch_unwind(AssertUnwindSafe(|| {
            #[cfg(feature = "native-test-controls")]
            {
                if let Work::ArmTest(control) = work {
                    return controls.arm(control).map(|_| WorkerValue::Complete).unwrap_or_else(WorkerValue::Failure);
                }
                if matches!(work, Work::Resolve(_)) && controls.resolve(controls_stop).is_err() {
                    return WorkerValue::Failure(NativeFailure::Access);
                }
                if matches!(&work, Work::Write { job, .. } if matches!(job.token().purpose(), sketch_core::application::storage::SavePurpose::Save | sketch_core::application::storage::SavePurpose::SaveAs)) {
                    return state.perform(work, &controls.hooks(controls_stop));
                }
            }
            state.perform(work, &NoFaults)
        }))
            .unwrap_or(WorkerValue::Failure(NativeFailure::Panic));
        state.operation_event(
            operation,
            correlation,
            subsystem,
            kind,
            Some(&value),
            started.elapsed().as_micros().min(u64::MAX as u128) as u64,
        );
        if output.send(state.result(operation, value)).is_err() {
            break;
        }
    }
}
