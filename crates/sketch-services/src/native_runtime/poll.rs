use super::{NativeRuntime, owner_types::*, work::*};
impl NativeRuntime {
    pub fn state(&self) -> Result<RuntimeState, RuntimeError> {
        self.check_owner()?;
        let now = self.started.elapsed().as_millis() as u64;
        let wake = self
            .sessions
            .iter()
            .filter(|slot| !slot.detached && slot.recovery.as_ref().is_some_and(|r| r.state == 2))
            .filter_map(|slot| slot.schedule.wake_after_ms(now))
            .min();
        Ok(RuntimeState {
            pending: self.operations.len(),
            ready: self.events.len(),
            retained: self.delivered.len(),
            attached: self.sessions.iter().filter(|s| !s.detached).count(),
            needs_idle: wake.is_some(),
            stopping: self.stopping,
            stopped: self.stopping && self.io.stopped() && self.shutdown_notified,
            wake_after_ms: wake.map_or(u64::MAX, |ms| ms.max(16)),
            diagnostic_drops: self.drops,
            diagnostic_sink_failures: self.sink_failures,
        })
    }
    pub fn poll(&mut self, capacity: usize) -> Result<Vec<NativeEvent>, RuntimeError> {
        self.check_owner()?;
        if capacity == 0 {
            return Ok(Vec::new());
        }
        for _ in 0..32 {
            match self.io.poll() {
                Ok(Some(result)) => self.receive(result)?,
                Ok(None) => break,
                Err(_) if self.stopping => break,
                Err(_) => return Err(RuntimeError::Stopped),
            }
        }
        self.drain_detached()?;
        self.pump()?;
        self.shutdown_progress()?;
        let count = capacity.min(32).min(self.events.len());
        let events: Vec<_> = self.events.drain(..count).collect();
        self.delivered.extend(events.iter().copied());
        Ok(events)
    }
    fn receive(&mut self, result: WorkerResult) -> Result<(), RuntimeError> {
        self.drops = result.diagnostic_drops;
        self.sink_failures = result.diagnostic_sink_failures;
        if let WorkerValue::Started { recovery, .. } = result.value {
            self.recovery_available = recovery.is_ok();
            return Ok(());
        }
        self.worker_active = false;
        let index = self
            .operations
            .iter()
            .position(|op| op.id == result.operation)
            .ok_or(RuntimeError::Invalid)?;
        if matches!(self.operations[index].kind, OperationKind::Retirement) {
            self.operations.remove(index);
            return Ok(());
        }
        let value = if let WorkerValue::Write(mut receipt) = result.value {
            let token = match self.operations[index].kind {
                OperationKind::Save(token) => token,
                _ => return Err(RuntimeError::Invalid),
            };
            if let Err(rejected) = self.coordinator.finish(receipt.write) {
                receipt.write = rejected.into_receipt();
                self.operations[index].state =
                    OperationState::Ready(Box::new(WorkerValue::Write(receipt)));
                // FFI catches this invariant breach and poisons the owner; retain the
                // issued receipt until explicit terminal cleanup, never fabricate success.
                panic!("native receipt violated the coordinator contract");
            }
            let completion = self
                .coordinator
                .take_completion(token)
                .map_err(|_| RuntimeError::Invalid)?;
            WorkerValue::Completed(Some(completion), receipt.report)
        } else {
            result.value
        };
        let value = match value {
            WorkerValue::Replacement(result) => match *result {
                Ok(candidate) => WorkerValue::Prepared(Box::new(candidate)),
                Err(failure) => WorkerValue::ReplacementFailed(Some(failure)),
            },
            other => other,
        };
        self.registration_result(index, &value);
        let items = if let WorkerValue::Inventory(listing) = &value {
            let mut items = Vec::new();
            for metadata in &listing.candidates {
                items.push((self.id()?, metadata.clone()));
            }
            items
        } else {
            Vec::new()
        };
        let event_id = self.id()?;
        let op = &mut self.operations[index];
        let failure = match &value {
            WorkerValue::Failure(failure) => Some(*failure),
            _ => None,
        };
        let kind = match op.kind {
            OperationKind::List => EventKind::RecoveryReady,
            OperationKind::Register(_)
            | OperationKind::RetryRecovery(_)
            | OperationKind::Maintenance => EventKind::Maintenance,
            _ => EventKind::OwnerReady,
        };
        let event = NativeEvent {
            operation_kind: op.boundary,
            id: event_id,
            operation: op.id,
            window: op.window,
            session: op.session,
            request: op.request,
            grant: op.grant,
            kind,
            failure,
        };
        op.items = items;
        op.event = Some(event_id);
        op.state = OperationState::Ready(Box::new(value));
        self.events.push_back(event);
        Ok(())
    }
    pub(super) fn pump(&mut self) -> Result<(), RuntimeError> {
        if self.worker_active {
            return Ok(());
        }
        if self.barrier.is_some_and(|id| {
            self.operations
                .iter()
                .any(|op| op.id == id && matches!(op.state, OperationState::Ready(_)))
        }) {
            return Ok(());
        }
        let Some(index) = self.operations.iter().position(|op| {
            matches!(
                op.state,
                OperationState::Queued(_) | OperationState::WaitingWrite(_)
            )
        }) else {
            return Ok(());
        };
        let state = std::mem::replace(&mut self.operations[index].state, OperationState::Running);
        let work = match state {
            OperationState::Queued(work) => *work,
            OperationState::WaitingWrite(token) => {
                let job = self.coordinator.start_next().ok_or(RuntimeError::Invalid)?;
                if job.token() != token {
                    return Err(RuntimeError::Invalid);
                }
                Work::Write {
                    job,
                    checkpoint: self.operations[index].checkpoint.clone(),
                }
            }
            _ => unreachable!(),
        };
        let op = &self.operations[index];
        let request = WorkerRequest {
            correlation: Correlation {
                session: op.session,
                request: op.request,
                revision: match &work {
                    Work::Write { job, .. } => job.token().revision().0,
                    Work::Prepare { ticket, .. } => ticket.revision().0,
                    _ => 0,
                },
            },
            operation: self.operations[index].id,
            work,
        };
        match self.io.submit(request) {
            Ok(()) => self.worker_active = true,
            Err(request) => {
                self.operations[index].state = OperationState::Queued(Box::new(request.work))
            }
        }
        Ok(())
    }
}
