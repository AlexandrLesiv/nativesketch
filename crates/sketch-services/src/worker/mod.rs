mod run;
use crate::diagnostics::Event;
use crate::recovery::{CheckpointMetadata, RecoveryStore};
use crate::storage::{DestinationAccess, NoFaults, StoreHooks, StoreReport};
use sketch_core::application::storage::{WriteJob, WriteReceipt};
use std::io;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
    mpsc::{self, Receiver, SyncSender, TrySendError},
};

pub enum WriteTarget {
    Document(DestinationAccess),
    Checkpoint {
        store: RecoveryStore,
        metadata: CheckpointMetadata,
    },
}
pub struct Submission {
    pub job: WriteJob,
    pub target: WriteTarget,
}
pub struct ServiceReceipt {
    pub write: WriteReceipt,
    pub report: StoreReport,
    pub events: Vec<Event>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorkerDisconnected;
/// Native bounded serial worker. Submit/poll/shutdown request and Drop never join or wait.
pub struct SerialStorageWorker {
    sender: Option<SyncSender<Box<Submission>>>,
    receiver: Receiver<ServiceReceipt>,
    stop: Arc<AtomicBool>,
    stopped: Arc<AtomicBool>,
}
impl SerialStorageWorker {
    pub fn spawn(build: u64) -> io::Result<Self> {
        Self::with_hooks(build, Arc::new(NoFaults))
    }
    pub fn with_hooks(build: u64, hooks: Arc<dyn StoreHooks>) -> io::Result<Self> {
        let (sender, input) = mpsc::sync_channel(1);
        let (output, receiver) = mpsc::sync_channel(1);
        let stop = Arc::new(AtomicBool::new(false));
        let stopped = Arc::new(AtomicBool::new(false));
        let thread_stop = Arc::clone(&stop);
        let thread_stopped = Arc::clone(&stopped);
        std::thread::Builder::new()
            .name("sketch-storage".into())
            .spawn(move || {
                run::worker(input, output, &thread_stop, build, hooks.as_ref());
                thread_stopped.store(true, Ordering::Release);
            })?;
        Ok(Self {
            sender: Some(sender),
            receiver,
            stop,
            stopped,
        })
    }
    /// Full/disconnected returns unique ownership intact; caller must complete/retry the job.
    pub fn try_submit(&self, submission: Submission) -> Result<(), Box<Submission>> {
        let submission = Box::new(submission);
        if self.stop.load(Ordering::Acquire) {
            return Err(submission);
        }
        match self
            .sender
            .as_ref()
            .map(|sender| sender.try_send(submission))
        {
            Some(Ok(())) => Ok(()),
            Some(Err(TrySendError::Full(job) | TrySendError::Disconnected(job))) => Err(job),
            None => unreachable!("sender retained while self is alive"),
        }
    }
    pub fn poll_receipt(&self) -> Result<Option<ServiceReceipt>, WorkerDisconnected> {
        match self.receiver.try_recv() {
            Ok(receipt) => Ok(Some(receipt)),
            Err(mpsc::TryRecvError::Empty) => Ok(None),
            Err(mpsc::TryRecvError::Disconnected) => Err(WorkerDisconnected),
        }
    }
    /// Drain receipts until is_stopped; an active write completes, it is never rolled back.
    pub fn request_shutdown(&mut self) {
        self.stop.store(true, Ordering::Release);
        self.sender.take();
    }
    pub fn is_stopped(&self) -> bool {
        self.stopped.load(Ordering::Acquire)
    }
}
impl Drop for SerialStorageWorker {
    fn drop(&mut self) {
        self.request_shutdown();
    }
}
