use crate::{Runtime, dto, recovery_api::sketch_runtime_v1_recovery_copy};
use sketch_services::{
    native_runtime::{EventKind, NativeRuntime, RuntimeConfig},
    recovery::{CheckpointMetadata, RecoveryEnvelope, RecoveryId, RecoveryLimits, RecoveryStore},
    storage::NoFaults,
};
use std::{
    fs,
    time::{Duration, Instant},
};

fn ready(runtime: &mut NativeRuntime) -> sketch_services::native_runtime::NativeEvent {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if let Some(event) = runtime.poll(1).unwrap().first() {
            return *event;
        }
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(1));
    }
}

fn copy(runtime: &mut Runtime) -> (u64, Vec<dto::RecoveryItem>, dto::RecoveryInfo) {
    runtime.core.list_recoveries(1).unwrap();
    let event = ready(&mut runtime.core);
    assert!(matches!(event.kind, EventKind::RecoveryReady));
    let mut items = Vec::<dto::RecoveryItem>::with_capacity(20);
    let mut count = 0;
    let mut info = dto::RecoveryInfo::default();
    // The runtime and output allocations are uniquely owned and valid for the call.
    unsafe {
        assert_eq!(
            sketch_runtime_v1_recovery_copy(
                runtime,
                event.id,
                items.as_mut_ptr(),
                20,
                &mut count,
                &mut info
            ),
            0
        );
        assert!(count <= 20);
        items.set_len(count as usize);
    }
    (event.id, items, info)
}

#[test]
fn recovery_copy_projects_worker_capacity_and_distinguishes_known_empty_usage() {
    for amount in [0, 20] {
        let root = fs::canonicalize(std::env::temp_dir())
            .unwrap()
            .join(format!(
                "native-recovery-info-{}-{amount}",
                std::process::id()
            ));
        // Open provisions the private parent; the runtime owns its sessions child.
        RecoveryStore::open(root.clone(), RecoveryLimits::default()).unwrap();
        let store = RecoveryStore::open(root.join("sessions"), RecoveryLimits::default()).unwrap();
        let mut expected_bytes = 0;
        for number in 1..=amount {
            let identity = RecoveryId::from_bytes([number; 16]).unwrap();
            store.allocate(identity).unwrap();
            let envelope = RecoveryEnvelope::new(
                CheckpointMetadata {
                    identity,
                    generation: 1,
                    revision: 1,
                    created_seconds: 1,
                    display_name: "Fixture".into(),
                },
                include_bytes!("../../sketch-core/tests/fixtures/portable-v1.nsketch").to_vec(),
            )
            .unwrap();
            expected_bytes += envelope.encode().len() as u64;
            store.publish(envelope, &NoFaults).unwrap();
        }
        let mut runtime = Runtime {
            core: NativeRuntime::new(RuntimeConfig {
                recovery_root: Some(root.clone()),
                ..Default::default()
            })
            .unwrap(),
            owner: std::thread::current().id(),
            poisoned: false,
        };
        let (event, items, info) = copy(&mut runtime);
        assert_eq!(info.count, amount.into());
        assert_eq!(info.usage_known, 1);
        assert_eq!(info.used_bytes, expected_bytes);
        assert_eq!(info.limit_bytes, RecoveryLimits::default().bytes);
        assert_eq!(info.capacity_blocked, u32::from(amount == 20));
        assert_eq!(info.rejected_sessions, 0);
        if amount == 20 {
            runtime.core.discard_recovery(2, items[0].item_id).unwrap();
            let discarded = ready(&mut runtime.core);
            assert!(discarded.failure.is_none());
            runtime.core.release_event(discarded.id).unwrap();
            runtime.core.release_event(event).unwrap();
            let (event, _, info) = copy(&mut runtime);
            assert_eq!(info.count, 19);
            assert_eq!(info.usage_known, 1);
            assert_eq!(info.capacity_blocked, 0);
            assert!(info.used_bytes < expected_bytes);
            runtime.core.release_event(event).unwrap();
        } else {
            runtime.core.release_event(event).unwrap();
            let foreign = root.join("sessions/foreign-entry");
            fs::write(&foreign, b"not a recovery session").unwrap();
            let (event, _, info) = copy(&mut runtime);
            assert_eq!(info.count, 0);
            assert_eq!(info.usage_known, 0);
            assert_eq!(info.used_bytes, 0);
            assert_eq!(info.rejected_sessions, 1);
            assert_eq!(fs::read(foreign).unwrap(), b"not a recovery session");
            runtime.core.release_event(event).unwrap();
        }
        runtime.core.request_shutdown().unwrap();
        let deadline = Instant::now() + Duration::from_secs(5);
        while !runtime.core.free_ready().unwrap() {
            for event in runtime.core.poll(32).unwrap() {
                runtime.core.release_event(event.id).unwrap();
            }
            assert!(Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(1));
        }
        drop(runtime);
        fs::remove_dir_all(root).unwrap();
    }
}
