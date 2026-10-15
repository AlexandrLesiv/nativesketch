use sketch_services::native_runtime::{NativeIo, RuntimeConfig, work::*};
use sketch_services::recovery::RecoveryId;
use std::{
    fs,
    path::PathBuf,
    time::{Duration, Instant},
};
static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
fn root() -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "native-worker-{}-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir(&path).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&path, fs::Permissions::from_mode(0o700)).unwrap();
    }
    path.canonicalize().unwrap()
}
fn receive(worker: &NativeIo) -> WorkerResult {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if let Some(result) = worker.poll().expect("worker alive") {
            return result;
        }
        assert!(Instant::now() < deadline, "worker timed out");
        std::thread::sleep(Duration::from_millis(1));
    }
}
fn stop(worker: &mut NativeIo) {
    worker.stop();
    let deadline = Instant::now() + Duration::from_secs(5);
    while !worker.stopped() {
        let _ = worker.poll();
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(1));
    }
}
#[test]
fn exclusive_recovery_root_degrades_second_runtime_without_blocking_local_io() {
    let root = root();
    let config = RuntimeConfig {
        recovery_root: Some(root.clone()),
        ..RuntimeConfig::default()
    };
    let mut first = NativeIo::spawn(config.clone()).unwrap();
    assert!(matches!(
        receive(&first).value,
        WorkerValue::Started {
            recovery: Ok(()),
            ..
        }
    ));
    let mut second = NativeIo::spawn(config).unwrap();
    assert!(matches!(
        receive(&second).value,
        WorkerValue::Started {
            recovery: Err(_),
            ..
        }
    ));
    assert!(
        second
            .submit(WorkerRequest {
                operation: 1,
                work: Work::Resolve(root.join("drawing.nsketch"))
            })
            .is_ok()
    );
    assert!(matches!(
        receive(&second).value,
        WorkerValue::Destination(_)
    ));
    stop(&mut first);
    assert!(
        second
            .submit(WorkerRequest {
                operation: 2,
                work: Work::Retry
            })
            .is_ok()
    );
    assert!(matches!(receive(&second).value, WorkerValue::Complete));
    stop(&mut second);
    fs::remove_dir_all(root).unwrap();
}
#[test]
fn disabled_recovery_is_observable_and_ordered_registration_can_be_listed() {
    let root = root();
    let mut disabled = NativeIo::spawn(RuntimeConfig::default()).unwrap();
    assert!(matches!(
        receive(&disabled).value,
        WorkerValue::Started {
            recovery: Err(NativeFailure::Unsupported),
            ..
        }
    ));
    stop(&mut disabled);
    let mut worker = NativeIo::spawn(RuntimeConfig {
        recovery_root: Some(root.clone()),
        ..RuntimeConfig::default()
    })
    .unwrap();
    receive(&worker);
    assert!(
        worker
            .submit(WorkerRequest {
                operation: 1,
                work: Work::Register(RecoveryId::from_bytes([7; 16]).unwrap())
            })
            .is_ok()
    );
    assert!(matches!(receive(&worker).value, WorkerValue::Complete));
    assert!(
        worker
            .submit(WorkerRequest {
                operation: 2,
                work: Work::List
            })
            .is_ok()
    );
    match receive(&worker).value {
        WorkerValue::Inventory(list) => assert!(list.candidates.is_empty()),
        _ => panic!("expected inventory"),
    }
    stop(&mut worker);
    fs::remove_dir_all(root).unwrap();
}
