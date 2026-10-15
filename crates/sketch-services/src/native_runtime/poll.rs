use super::{NativeRuntime, owner_types::*, work::*};
impl NativeRuntime {
    pub fn state(&self) -> Result<RuntimeState, RuntimeError> {
        self.check_owner()?;
        let recovery_pending = self
            .sessions
            .iter()
            .any(|slot| slot.recovery.as_ref().is_some_and(|r| r.state == 2));
        Ok(RuntimeState {
            pending: self.operations.len(),
            ready: self.events.len(),
            retained: self.delivered.len(),
            attached: self.sessions.iter().filter(|s| !s.detached).count(),
            needs_idle: recovery_pending,
            stopping: self.stopping,
            stopped: self.stopping && self.io.stopped(),
            wake_after_ms: if recovery_pending { 250 } else { u64::MAX },
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
        let value = if let WorkerValue::Write(receipt) = result.value {
            let token = match self.operations[index].kind {
                OperationKind::Save(token) => token,
                _ => return Err(RuntimeError::Invalid),
            };
            self.coordinator
                .finish(receipt.write)
                .map_err(|_| RuntimeError::Invalid)?;
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
            OperationKind::Register | OperationKind::RetryRecovery | OperationKind::Maintenance => {
                EventKind::Maintenance
            }
            _ => EventKind::OwnerReady,
        };
        let event = NativeEvent {
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
        let request = WorkerRequest {
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
