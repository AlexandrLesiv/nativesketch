use super::{RuntimeConfig, work::*};
use std::io;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
    mpsc::{self, Receiver, SyncSender, TrySendError},
};
pub struct NativeIo {
    sender: Option<SyncSender<Box<WorkerRequest>>>,
    receiver: Receiver<WorkerResult>,
    stop: Arc<AtomicBool>,
    stopped: Arc<AtomicBool>,
}
impl NativeIo {
    pub fn spawn(config: RuntimeConfig) -> io::Result<Self> {
        if !config.validate() {
            return Err(io::ErrorKind::InvalidInput.into());
        }
        let (sender, input) = mpsc::sync_channel(1);
        let (output, receiver) = mpsc::sync_channel(1);
        let stop = Arc::new(AtomicBool::new(false));
        let stopped = Arc::new(AtomicBool::new(false));
        let worker_stop = Arc::clone(&stop);
        let worker_stopped = Arc::clone(&stopped);
        std::thread::Builder::new()
            .name("sketch-native-io".into())
            .spawn(move || {
                super::io_run::run(config, input, output, &worker_stop);
                worker_stopped.store(true, Ordering::Release);
            })?;
        Ok(Self {
            sender: Some(sender),
            receiver,
            stop,
            stopped,
        })
    }
    pub fn submit(&self, request: WorkerRequest) -> Result<(), Box<WorkerRequest>> {
        let request = Box::new(request);
        let Some(sender) = self.sender.as_ref() else {
            return Err(request);
        };
        match sender.try_send(request) {
            Ok(()) => Ok(()),
            Err(TrySendError::Full(request) | TrySendError::Disconnected(request)) => Err(request),
        }
    }
    pub fn poll(&self) -> Result<Option<WorkerResult>, NativeFailure> {
        match self.receiver.try_recv() {
            Ok(result) => Ok(Some(result)),
            Err(mpsc::TryRecvError::Empty) => Ok(None),
            Err(mpsc::TryRecvError::Disconnected) => Err(NativeFailure::Stopped),
        }
    }
    pub fn stop(&mut self) {
        self.stop.store(true, Ordering::Release);
        self.sender.take();
    }
    pub fn stopped(&self) -> bool {
        self.stopped.load(Ordering::Acquire)
    }
}
impl Drop for NativeIo {
    fn drop(&mut self) {
        self.stop();
    }
}
