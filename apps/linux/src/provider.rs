use crate::{
    command_owner::CommandOwner,
    provider_work::{Job, execute},
};
use sketch_app::*;
use sketch_core::application::{CommandContext, routing::EffectToken};
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{
        atomic::{AtomicU64, Ordering},
        mpsc::{self, Receiver, SyncSender},
    },
    thread::JoinHandle,
};
static NEXT_RESOURCE: AtomicU64 = AtomicU64::new(1);
pub(crate) struct Provider {
    leases: HashMap<u64, (ResourceDescriptor, PathBuf)>,
    sender: Option<SyncSender<Job>>,
    receiver: Receiver<WorkResult>,
    worker: Option<JoinHandle<()>>,
    pending: Option<Job>,
    active: bool,
    completed: Option<WorkResult>,
    ready: Option<WorkInfo>,
}
impl Provider {
    pub fn new() -> Self {
        let (sender, jobs) = mpsc::sync_channel::<Job>(1);
        let (completed, receiver) = mpsc::sync_channel(1);
        let worker = std::thread::spawn(move || {
            while let Ok(job) = jobs.recv() {
                let result = execute(job);
                if completed.send(result).is_err() {
                    break;
                }
            }
        });
        Self {
            leases: HashMap::new(),
            sender: Some(sender),
            receiver,
            worker: Some(worker),
            pending: None,
            active: false,
            completed: None,
            ready: None,
        }
    }
    pub fn supply(
        &mut self,
        owner: &mut CommandOwner,
        context: CommandContext,
        token: EffectToken,
        path: PathBuf,
    ) -> Result<(), AppError> {
        let id = NEXT_RESOURCE
            .try_update(Ordering::Relaxed, Ordering::Relaxed, |id| id.checked_add(1))
            .map_err(|_| AppError::Capacity)?;
        let descriptor = ResourceDescriptor {
            provider: ProviderId(1),
            resource: ProviderResourceId(id),
            canonical: ProviderResourceId(id),
            generation: id,
            access: ResourceAccess {
                read: true,
                write: true,
                confirmed_write: true,
            },
        };
        // No filesystem lookup on GTK owner. Unknown aliases stay ordered by PortableApp.
        let outcome = owner
            .app
            .supply_resource(owner.session, context, token, descriptor)?;
        self.leases.insert(id, (descriptor, path));
        owner.accept(outcome);
        Ok(())
    }
    pub fn poll(&mut self, owner: &mut CommandOwner) -> Result<(), AppError> {
        if self.completed.is_none() && self.ready.is_none() {
            self.completed = self.receiver.try_recv().ok();
        }
        if let Some(result) = self.completed.as_ref() {
            let info = result.info();
            owner.app.complete_work(&mut self.completed)?;
            self.active = false;
            if info.kind != WorkKind::Retire {
                self.ready = Some(info);
            }
        }
        if let Some(info) = self.ready {
            match owner.app.apply_ready(info.session, info.operation) {
                Ok(outcome) => {
                    self.ready = None;
                    owner.accept(outcome);
                }
                Err(AppError::Core(error))
                    if matches!(
                        error.failure,
                        sketch_core::application::routing::ApplicationFailure::InvalidEffect
                            | sketch_core::application::routing::ApplicationFailure::StaleDocument
                    ) =>
                {
                    self.ready = None;
                    crate::diagnostics::record(format_args!(
                        "event=stale_completion operation={:?}",
                        info.operation
                    ));
                }
                Err(error) => return Err(error),
            }
        }
        while let Some(release) = owner.app.take_release() {
            let key = release.descriptor.resource.0;
            if self
                .leases
                .get(&key)
                .is_some_and(|(descriptor, _)| *descriptor == release.descriptor)
            {
                self.leases.remove(&key);
            } else {
                return Err(AppError::Stale);
            }
        }
        while let Some(diagnostic) = owner.app.take_diagnostic() {
            match diagnostic.outcome {
                DiagnosticOutcome::WorkCompleted
                | DiagnosticOutcome::Write(
                    sketch_core::application::storage::SaveResult::Written(_),
                ) => crate::diagnostics::event(format_args!(
                    "event=provider_boundary diagnostic={diagnostic:?}"
                )),
                _ => crate::diagnostics::record(format_args!(
                    "event=provider_boundary diagnostic={diagnostic:?}"
                )),
            }
        }
        // One active job plus one retained host slot; ownership is never dropped on send error.
        if !self.active
            && self.pending.is_none()
            && self.sender.is_some()
            && let Some(work) = owner.app.take_work()?
        {
            let path = work.info().resource.and_then(|resource| {
                self.leases
                    .get(&resource.resource.0)
                    .filter(|(descriptor, _)| *descriptor == resource)
                    .map(|(_, path)| path.clone())
            });
            self.pending = Some(Job { work, path });
        }
        if let Some(job) = self.pending.take() {
            match self
                .sender
                .as_ref()
                .expect("live worker sender")
                .try_send(job)
            {
                Ok(()) => self.active = true,
                Err(error) => {
                    self.pending = Some(match error {
                        mpsc::TrySendError::Full(job) | mpsc::TrySendError::Disconnected(job) => {
                            job
                        }
                    });
                    return Err(AppError::Busy);
                }
            }
        }
        Ok(())
    }
    pub fn stop_when_drained(&mut self, owner: &CommandOwner) -> bool {
        if !owner.app.stats().shutdown_ready()
            || self.active
            || self.pending.is_some()
            || self.completed.is_some()
            || self.ready.is_some()
            || !self.leases.is_empty()
        {
            return false;
        }
        self.sender.take();
        if self.worker.as_ref().is_some_and(JoinHandle::is_finished) {
            if self.worker.take().unwrap().join().is_err() {
                crate::diagnostics::record(format_args!("event=provider_worker_failed"));
            }
            return true;
        }
        false
    }
}
