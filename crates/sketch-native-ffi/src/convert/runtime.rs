use crate::dto;
use sketch_services::native_runtime::{EventKind, NativeEvent, RuntimeState};
pub fn state(value: RuntimeState) -> dto::RuntimeState {
    dto::RuntimeState {
        pending_operations: value.pending as u32,
        ready_events: value.ready as u32,
        retained_events: value.retained as u32,
        attached_sessions: value.attached as u32,
        needs_poll: u32::from(
            value.pending != 0 || value.ready != 0 || value.stopping && !value.stopped,
        ),
        needs_idle: value.needs_idle.into(),
        stopping: value.stopping.into(),
        stopped: value.stopped.into(),
        wake_after_ms: value.wake_after_ms,
        dropped_diagnostics: value.diagnostic_drops,
        failed_diagnostic_sinks: value.diagnostic_sink_failures,
    }
}
pub fn event(value: NativeEvent) -> dto::Event {
    let mut failure = value
        .failure
        .map(super::failures::native)
        .unwrap_or_default();
    failure.operation = value.operation_kind;
    failure.session = value.session;
    failure.request = value.request;
    failure.operation_id = value.operation;
    dto::Event {
        operation_id: value.operation,
        event_id: value.id,
        window: value.window,
        session: value.session,
        request: value.request,
        grant_id: value.grant,
        kind: match value.kind {
            EventKind::OwnerReady => 1,
            EventKind::GrantRelease => 2,
            EventKind::RecoveryReady => 3,
            EventKind::Maintenance => 4,
            EventKind::Failure => 5,
            EventKind::Shutdown => 6,
        },
        reserved: 0,
        failure,
    }
}
