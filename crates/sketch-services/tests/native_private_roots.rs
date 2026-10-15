use sketch_services::native_runtime::{NativeIo, RuntimeConfig, work::*};
use std::{
    fs,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};
static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
fn fixture() -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "native-private-roots-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ));
    fs::create_dir(&root).unwrap();
    root.canonicalize().unwrap()
}
fn inspect(root: &Path) -> WorkerValue {
    let mut worker = NativeIo::spawn(RuntimeConfig {
        recovery_root: Some(root.join("Recovery")),
        diagnostics_root: Some(root.join("Diagnostics")),
        ..Default::default()
    })
    .unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    let value = loop {
        if let Some(result) = worker.poll().unwrap() {
            break result.value;
        }
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(1));
    };
    worker.stop();
    while !worker.stopped() {
        let _ = worker.poll();
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(1));
    }
    value
}
#[test]
fn fresh_nested_roots_are_provisioned_privately_by_worker_and_reopen() {
    let fixture = fixture();
    let root = fixture.join("new-application/runtime");
    assert!(!root.exists());
    assert!(matches!(
        inspect(&root),
        WorkerValue::Started {
            recovery: Ok(()),
            diagnostics: Ok(())
        }
    ));
    assert!(root.join("Recovery/sessions").is_dir());
    assert!(root.join("Diagnostics/diagnostics.0.jsonl").is_file());
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        for path in [
            fixture.join("new-application"),
            root.clone(),
            root.join("Recovery"),
            root.join("Diagnostics"),
        ] {
            assert_eq!(
                fs::metadata(path).unwrap().permissions().mode() & 0o777,
                0o700
            );
        }
    }
    assert!(matches!(
        inspect(&root),
        WorkerValue::Started {
            recovery: Ok(()),
            diagnostics: Ok(())
        }
    ));
    fs::remove_dir_all(fixture).unwrap();
}
#[test]
fn non_directory_ancestor_is_rejected_without_overwrite() {
    let fixture = fixture();
    let blocked = fixture.join("blocked");
    fs::write(&blocked, b"retain").unwrap();
    assert!(matches!(
        inspect(&blocked.join("runtime")),
        WorkerValue::Started {
            recovery: Err(NativeFailure::Access),
            diagnostics: Err(NativeFailure::Access)
        }
    ));
    assert_eq!(fs::read(blocked).unwrap(), b"retain");
    fs::remove_dir_all(fixture).unwrap();
}
#[cfg(unix)]
#[test]
fn symlink_ancestors_and_permissive_existing_roots_are_rejected_unchanged() {
    use std::os::unix::fs::{PermissionsExt, symlink};
    let fixture = fixture();
    let target = fixture.join("target");
    fs::create_dir(&target).unwrap();
    let alias = fixture.join("alias");
    symlink(&target, &alias).unwrap();
    assert!(matches!(
        inspect(&alias.join("runtime")),
        WorkerValue::Started {
            recovery: Err(NativeFailure::Access),
            diagnostics: Err(NativeFailure::Access)
        }
    ));
    assert_eq!(fs::read_dir(&target).unwrap().count(), 0);
    let root = fixture.join("existing");
    fs::create_dir(&root).unwrap();
    for leaf in ["Recovery", "Diagnostics"] {
        let path = root.join(leaf);
        fs::create_dir(&path).unwrap();
        fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
    }
    assert!(matches!(
        inspect(&root),
        WorkerValue::Started {
            recovery: Err(NativeFailure::Access),
            diagnostics: Err(NativeFailure::Access)
        }
    ));
    for leaf in ["Recovery", "Diagnostics"] {
        let path = root.join(leaf);
        assert_eq!(
            fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o755
        );
        assert_eq!(fs::read_dir(path).unwrap().count(), 0);
    }
    fs::remove_dir_all(fixture).unwrap();
}
