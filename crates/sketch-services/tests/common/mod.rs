#![allow(dead_code)]
use sketch_core::Brush;
use sketch_core::application::storage::{
    DestinationId, SaveCoordinator, SavePurpose, StorageLimits, WriteJob,
};
use sketch_core::application::{Capabilities, EditorSession, RequestId, ToolId, ValidatedDocument};
use sketch_services::recovery::{CheckpointMetadata, RecoveryEnvelope, RecoveryId};
use sketch_services::worker::{SerialStorageWorker, ServiceReceipt};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, Instant},
};
pub const FIXTURE: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../sketch-core/tests/fixtures/portable-v1.nsketch"
));
static NEXT: AtomicU64 = AtomicU64::new(0);
pub struct Temp(pub PathBuf);
impl Temp {
    pub fn new() -> Self {
        let path = fs::canonicalize(std::env::temp_dir())
            .unwrap()
            .join(format!(
                "sketch-service-test-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}
pub fn session() -> EditorSession {
    EditorSession::new(
        Capabilities {
            edit_document: true,
            export_document: true,
        },
        ToolId::Pen,
        Brush {
            rgba: [0.0, 0.0, 0.0, 1.0],
            width: 3.0,
        }
        .try_into()
        .unwrap(),
        Some(ValidatedDocument::decode(FIXTURE).unwrap()),
    )
    .unwrap()
}
pub fn job(purpose: SavePurpose) -> (EditorSession, SaveCoordinator, WriteJob) {
    let mut session = session();
    let mut queue = SaveCoordinator::new(StorageLimits {
        jobs: 2,
        snapshot_bytes: 1024 * 1024,
    })
    .unwrap();
    queue
        .request(&mut session, RequestId(42), purpose, DestinationId(1))
        .unwrap();
    let job = queue.start_next().unwrap();
    (session, queue, job)
}
pub fn receipt(worker: &SerialStorageWorker) -> ServiceReceipt {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if let Some(receipt) = worker
            .poll_receipt()
            .expect("worker disconnected before receipt")
        {
            return receipt;
        }
        assert!(Instant::now() < deadline, "worker receipt timed out");
        std::thread::sleep(Duration::from_millis(1));
    }
}
pub fn id(value: u8) -> RecoveryId {
    RecoveryId::from_bytes([value; 16]).unwrap()
}
pub fn metadata(generation: u64) -> CheckpointMetadata {
    CheckpointMetadata {
        identity: id(1),
        generation,
        revision: 1,
        created_seconds: generation,
        display_name: "Private drawing name".into(),
    }
}
pub fn envelope(generation: u64) -> RecoveryEnvelope {
    RecoveryEnvelope::new(metadata(generation), FIXTURE.to_vec()).unwrap()
}
