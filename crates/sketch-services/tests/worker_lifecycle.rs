mod common;
use common::*;
use sketch_core::application::storage::{Acknowledgement, SavePurpose};
use sketch_services::storage::{DestinationAccess, Stage, StoreHooks};
use sketch_services::worker::{SerialStorageWorker, Submission, WriteTarget};
use std::{
    io,
    sync::{Arc, Condvar, Mutex, mpsc},
    time::{Duration, Instant},
};
struct Gate {
    entered: mpsc::Sender<()>,
    released: Mutex<bool>,
    condition: Condvar,
}

#[test]
fn shutdown_drains_accepted_receipts_in_order_and_rejects_new_admission() {
    let temp = Temp::new();
    let (entered, receiver) = mpsc::channel();
    let gate = Arc::new(Gate {
        entered,
        released: Mutex::new(false),
        condition: Condvar::new(),
    });
    let mut worker = SerialStorageWorker::with_hooks(1, gate.clone()).unwrap();
    let mut accepted = Vec::new();
    for name in ["first", "second"] {
        let (session, queue, job) = job(SavePurpose::SaveAs);
        let token = job.token();
        assert!(
            worker
                .try_submit(Submission {
                    job,
                    target: WriteTarget::Document(
                        DestinationAccess::new(temp.0.join(name)).unwrap()
                    ),
                })
                .is_ok()
        );
        accepted.push((session, queue, token, name));
        if name == "first" {
            receiver.recv_timeout(Duration::from_secs(2)).unwrap();
        }
    }
    // A shutdown request must return while the first write is deliberately blocked.
    let (returned, shutdown) = mpsc::channel();
    std::thread::spawn(move || {
        worker.request_shutdown();
        returned.send(worker).ok();
    });
    let result = shutdown.recv_timeout(Duration::from_secs(2));
    *gate.released.lock().unwrap() = true;
    gate.condition.notify_all();
    let worker = result.expect("shutdown waited for the active write");
    let rejected = submission(&temp, "third");
    let rejected_token = rejected.job.token();
    assert_eq!(
        worker.try_submit(rejected).unwrap_err().job.token(),
        rejected_token
    );
    for (mut session, mut queue, token, name) in accepted {
        queue.finish(receipt(&worker).write).unwrap();
        assert_eq!(
            session.acknowledge_save(queue.take_completion(token).unwrap()),
            Acknowledgement::SavedCurrent
        );
        assert_eq!(std::fs::read(temp.0.join(name)).unwrap(), FIXTURE);
    }
    let deadline = Instant::now() + Duration::from_secs(2);
    while !worker.is_stopped() {
        assert!(
            Instant::now() < deadline,
            "shutdown did not finish after draining"
        );
        std::thread::sleep(Duration::from_millis(1));
    }
    assert!(worker.poll_receipt().is_err(), "unexpected extra receipt");
    assert!(!temp.0.join("third").exists());
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
