use sketch_services::native_runtime::{NativeIo, RuntimeConfig, work::*};
use std::{
    fs,
    path::PathBuf,
    process::{Command, Stdio},
    time::{Duration, Instant},
};
fn receive(worker: &NativeIo) -> WorkerResult {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if let Some(result) = worker.poll().unwrap() {
            return result;
        }
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(1));
    }
}
fn config(root: &std::path::Path) -> RuntimeConfig {
    RuntimeConfig {
        recovery_root: Some(root.join("recovery")),
        diagnostics_root: Some(root.join("logs")),
        ..Default::default()
    }
}
#[test]
#[ignore = "subprocess fixture executed by independent_root_locks_survive_conflict_and_release_on_process_death"]
fn child_holds_native_roots() {
    let root = PathBuf::from(std::env::var_os("SKETCH_ROOT_LOCK_FIXTURE").unwrap());
    let worker = NativeIo::spawn(config(&root)).unwrap();
    assert!(matches!(
        receive(&worker).value,
        WorkerValue::Started {
            recovery: Ok(()),
            diagnostics: Ok(())
        }
    ));
    fs::write(root.join("ready"), []).unwrap();
    loop {
        std::thread::park_timeout(Duration::from_secs(1));
    }
}
#[test]
fn independent_root_locks_survive_conflict_and_release_on_process_death() {
    let root = std::env::temp_dir().join(format!(
        "native-roots-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir(&root).unwrap();
    let root = root.canonicalize().unwrap();
    let mut child = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "child_holds_native_roots", "--ignored"])
        .env("SKETCH_ROOT_LOCK_FIXTURE", &root)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    while !root.join("ready").exists() && Instant::now() < deadline {
        if child.try_wait().unwrap().is_some() {
            panic!("child failed");
        }
        std::thread::sleep(Duration::from_millis(2));
    }
    let reached = root.join("ready").exists();
    if !reached {
        child.kill().unwrap();
        child.wait().unwrap();
        panic!("child timeout");
    }
    let mut contender = NativeIo::spawn(config(&root)).unwrap();
    let started = receive(&contender);
    child.kill().unwrap();
    child.wait().unwrap();
    assert!(matches!(
        started.value,
        WorkerValue::Started {
            recovery: Err(_),
            diagnostics: Err(_)
        }
    ));
    assert!(
        contender
            .submit(WorkerRequest {
                operation: 1,
                correlation: Default::default(),
                work: Work::Resolve(root.join("document.nsketch"))
            })
            .is_ok()
    );
    assert!(matches!(
        receive(&contender).value,
        WorkerValue::Destination(_)
    ));
    assert!(
        contender
            .submit(WorkerRequest {
                operation: 2,
                correlation: Default::default(),
                work: Work::Retry
            })
            .is_ok()
    );
    assert!(matches!(receive(&contender).value, WorkerValue::Complete));
    contender.stop();
    let deadline = Instant::now() + Duration::from_secs(5);
    while !contender.stopped() {
        let _ = contender.poll();
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(1));
    }
    let mut reopened = NativeIo::spawn(config(&root)).unwrap();
    assert!(matches!(
        receive(&reopened).value,
        WorkerValue::Started {
            recovery: Ok(()),
            diagnostics: Ok(())
        }
    ));
    reopened.stop();
    while !reopened.stopped() {
        let _ = reopened.poll();
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(1));
    }
    fs::remove_dir_all(root).unwrap();
}
