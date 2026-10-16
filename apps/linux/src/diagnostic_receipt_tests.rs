use crate::{
    provider_work::{Job, execute},
    test_support::Fixture,
};
use sketch_core::application::routing::FileIntent;
#[test]
fn broken_stderr_does_not_prevent_actual_worker_receipt() {
    let mut child = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "diagnostic_receipt_tests::broken_sink_provider_child",
            "--ignored",
        ])
        // Read-only descriptor: actual stderr writes fail with EBADF on supported Unix hosts.
        .stderr(std::fs::File::open("/dev/null").unwrap())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    while child.try_wait().unwrap().is_none() {
        if std::time::Instant::now() >= deadline {
            child.kill().unwrap();
            child.wait().unwrap();
            panic!("broken-sink probe timed out");
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
}
#[test]
#[ignore = "executed by broken_stderr_does_not_prevent_actual_worker_receipt with invalid stderr"]
fn broken_sink_provider_child() {
    crate::diagnostics::start();
    let mut f = Fixture::new();
    f.stroke();
    let path = f.directory.join("actual.nsketch");
    f.file(FileIntent::SaveAs);
    f.choose(path.clone());
    let work = f.model.owner.app.take_work().unwrap().unwrap();
    let info = work.info();
    let receipt = std::thread::spawn(move || {
        for _ in 0..1000 {
            crate::diagnostics::record(format_args!("event=worker_boundary"));
        }
        execute(Job {
            work,
            path: Some(path),
        })
    })
    .join()
    .expect("logging must not panic the provider worker");
    let mut receipt = Some(receipt);
    f.model.owner.app.complete_work(&mut receipt).unwrap();
    assert!(receipt.is_none());
    let outcome = f
        .model
        .owner
        .app
        .apply_ready(info.session, info.operation)
        .unwrap();
    f.model.owner.accept(outcome);
    assert!(!f.model.owner.state().dirty);
    f.drain();
}
