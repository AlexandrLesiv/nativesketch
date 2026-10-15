mod common;
use common::*;
use sketch_core::application::storage::SavePurpose;
use sketch_services::storage::{DestinationAccess, Stage, StoreHooks};
use sketch_services::worker::{SerialStorageWorker, Submission, WriteTarget};
use std::{
    io,
    sync::{Arc, Condvar, Mutex, mpsc},
    time::Duration,
};
struct Gate {
    entered: mpsc::Sender<()>,
    released: Mutex<bool>,
    condition: Condvar,
}
impl StoreHooks for Gate {
    fn at(&self, stage: Stage) -> io::Result<()> {
        if stage == Stage::BeforeCreate {
            self.entered.send(()).unwrap();
            let guard = self.released.lock().unwrap();
            drop(
                self.condition
                    .wait_while(guard, |released| !*released)
                    .unwrap(),
            );
        }
        Ok(())
    }
}
fn submission(temp: &Temp, name: &str) -> Submission {
    Submission {
        job: job(SavePurpose::SaveAs).2,
        target: WriteTarget::Document(DestinationAccess::new(temp.0.join(name)).unwrap()),
    }
}
#[test]
fn worker_backpressure_returns_unique_job_and_shutdown_does_not_join_active_write() {
    let temp = Temp::new();
    let (entered, receiver) = mpsc::channel();
    let gate = Arc::new(Gate {
        entered,
        released: Mutex::new(false),
        condition: Condvar::new(),
    });
    let worker = SerialStorageWorker::with_hooks(1, gate.clone()).unwrap();
    assert!(worker.try_submit(submission(&temp, "first")).is_ok());
    receiver.recv_timeout(Duration::from_secs(2)).unwrap();
    assert!(worker.try_submit(submission(&temp, "second")).is_ok());
    let third = submission(&temp, "third");
    let token = third.job.token();
    assert_eq!(worker.try_submit(third).unwrap_err().job.token(), token);
    let (dropped, did_drop) = mpsc::channel();
    std::thread::spawn(move || {
        drop(worker);
        dropped.send(()).unwrap();
    });
    let result = did_drop.recv_timeout(Duration::from_secs(2));
    *gate.released.lock().unwrap() = true;
    gate.condition.notify_all();
    assert!(result.is_ok(), "Drop waited for the blocked writer");
    // Wait for the detached worker's real write to finish before fixture destruction.
    for _ in 0..2000 {
        if Arc::strong_count(&gate) == 1 {
            assert!(temp.0.join("first").exists());
            return;
        }
        std::thread::sleep(Duration::from_millis(1));
    }
    panic!("writer did not finish after release");
}
